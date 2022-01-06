// Copyright 2021 the Gigamono authors. All rights reserved. Apache 2.0 license.

use crate::{events::Events, extensions, loaders, permissions::Permissions, RuntimeOptions};
use deno_core::{
    anyhow::Error,
    parking_lot::Mutex,
    v8::{self, Global, Value},
    Extension, JsRuntime, Snapshot,
};
use log::{debug, info};
use regex::Regex;
use std::fs;
use std::{cell::RefCell, path::PathBuf, rc::Rc};
use utilities::result::{Context, Result};

pub struct Runtime {
    runtime: JsRuntime,
    permissions: Rc<RefCell<Permissions>>,
}

impl Runtime {
    pub async fn new(
        permissions: Rc<RefCell<Permissions>>,
        enable_snapshot: bool,
        mut options: RuntimeOptions,
    ) -> Result<Self> {
        // TODO(appcypher): SEC: Support specifying maximum_heap_size_in_bytes.
        // TODO(appcypher): SEC: Add a callback that panics for near_heap_limit_callback.

        // Check if there is a startup snapshot.
        let has_startup_snapshot = options.startup_snapshot.is_some();

        debug!("Snapshot enabled = {}", enable_snapshot);
        debug!("Snapshot available = {}", has_startup_snapshot);

        // We create a new snapshot if snapshot is enabled but does not yet exist.
        if enable_snapshot && !has_startup_snapshot {
            debug!("Creating a snapshot runtime");

            // Get options tailored for snapshot.
            let snapshot_options = Self::create_snapshot_options(&options);

            // Create a temp runtime to prevent panic in the main runtime after creating a snapshot.
            let mut snapshot_runtime = JsRuntime::new(snapshot_options);

            // Execute postscripts and create snapshot.
            Self::execute_postscripts(&mut snapshot_runtime)?;
            Self::create_snapshot(&mut snapshot_runtime)?;

            // Update options with the new snapshot.
            options.startup_snapshot = Some(Snapshot::Boxed(Self::get_snapshot()));

            debug!("Dropping temp snapshot runtime");
        }

        // Create main runtime.
        let mut runtime = JsRuntime::new(options);

        if !enable_snapshot {
            debug!("Runtime will not use snapshot");
            Self::execute_postscripts(&mut runtime)?;
        }

        Ok(Self {
            runtime,
            permissions,
        })
    }

    pub async fn with_permissions(
        permissions: Permissions,
        enable_snapshot: bool,
        options: RuntimeOptions,
    ) -> Result<Self> {
        let permissions = Rc::new(RefCell::new(permissions));

        let mut startup_snapshot = None;
        if enable_snapshot && Self::snapshot_exists() {
            startup_snapshot = Some(Snapshot::Boxed(Self::get_snapshot()));
        }

        // Set runtime options
        let opts = RuntimeOptions {
            module_loader: Some(Rc::new(loaders::esm(Rc::clone(&permissions)))),
            extensions: vec![extensions::fs(Rc::clone(&permissions))],
            startup_snapshot,
            ..options
        };

        Self::new(permissions, enable_snapshot, opts).await
    }

    pub async fn with_events(
        permissions: Permissions,
        events: Rc<RefCell<Events>>,
        enable_snapshot: bool,
        options: RuntimeOptions,
    ) -> Result<Self> {
        let permissions = Rc::new(RefCell::new(permissions));

        let mut startup_snapshot = None;
        if enable_snapshot && Self::snapshot_exists() {
            startup_snapshot = Some(Snapshot::Boxed(Self::get_snapshot()));
        }

        // Set runtime options
        let opts = RuntimeOptions {
            module_loader: Some(Rc::new(loaders::esm(Rc::clone(&permissions)))),
            extensions: vec![
                extensions::fs(Rc::clone(&permissions)),
                extensions::event_http(Rc::clone(&permissions), events),
            ],
            startup_snapshot,
            ..options
        };

        Self::new(permissions, enable_snapshot, opts).await
    }

    pub async fn execute_module(
        &mut self,
        filename: impl AsRef<str>,
        module_code: impl Into<String>,
    ) -> Result<()> {
        let filename = filename.as_ref();
        let module_code = module_code.into();

        // Add file scheme to filename and resolve to URL.
        let module_specifier =
            deno_core::resolve_url(&format!("file://{}", filename)).context(format!(
                r#"resolving main module specifier as "file://{}""#,
                filename
            ))?;

        // Load main module and deps.
        let module_id = self
            .runtime
            .load_main_module(&module_specifier, Some(module_code))
            .await
            .context("loading the main module")?;

        // Run main module.
        let mut rx = self.runtime.mod_evaluate(module_id);

        // Wait for message from module eval or event loop.
        tokio::select! {
            maybe_result = &mut rx => {
                Self::handle_reciever_error(maybe_result)?;

                // Continue event loop.
                self.runtime.run_event_loop(false).await.context("running the event loop")?;
            }
            event_loop_result = self.runtime.run_event_loop(false) => {
                event_loop_result.context("running the event loop")?;

                // Continue waiting on reciever.
                Self::handle_reciever_error(rx.await)?;
            }
        };

        Ok(())
    }

    pub async fn execute_middleware_script(
        &mut self,
        filename: impl AsRef<str>,
        script_code: impl AsRef<str>,
        permissions: Permissions,
    ) -> Result<Global<Value>> {
        // Replace existing permissions with new permissions.
        let existing_permissions = self.permissions.replace(permissions);

        // Execute script.
        let value = self
            .runtime
            .execute_script(filename.as_ref(), script_code.as_ref())
            .context("executing script")?;

        // Revert exising permissions.
        let _ = self.permissions.replace(existing_permissions);

        Ok(value)
    }

    pub fn handle_scope(&mut self) -> v8::HandleScope {
        self.runtime.handle_scope()
    }

    fn execute_postscripts(runtime: &mut JsRuntime) -> Result<()> {
        // TODO(appcypher): Need to make it possible for users to skip Tera's postscripts and add their own.
        // Get postcripts directory.
        let postscripts_dir =
            &PathBuf::from(concat!(env!("CARGO_MANIFEST_DIR"), "/lib/postscripts"));

        // Blindly assume everything in directory is a postscript.
        let mut postscripts = fs::read_dir(postscripts_dir)
            .context(format!(r#"reading postcripts dir "{:?}""#, postscripts_dir))?
            .map(|entry| -> Result<PathBuf> {
                Ok(entry
                    .context("collecting entries in postcripts dir")?
                    .path())
            })
            .collect::<Result<Vec<PathBuf>>>()?;

        // Sort postscripts.
        postscripts.sort();

        for path in postscripts.iter() {
            // Read content.
            let content = fs::read_to_string(&path)
                .context(format!(r#"getting postscript file, "{:?}""#, path))?;

            // Execute postscript.
            runtime
                .execute_script(&format!("(tera:postscripts) {}", path.display()), &content)
                .context("executing postscript file")?;
        }

        info!("Executed postscripts");

        Ok(())
    }

    fn create_snapshot_options(options: &RuntimeOptions) -> RuntimeOptions {
        // Construct the extensions. We only need the source pairs from the extensions. core/runtime.rs#init_extension_js
        let mut extensions: Vec<Extension> = vec![];
        for ext in &options.extensions {
            let mut js_files: Vec<(&'static str, Box<dyn Fn() -> Result<String, Error>>)> = vec![];

            for (filepath, _) in ext.init_js() {
                let filepath_rc = Rc::new(filepath.to_string());

                let pair: (&'static str, Box<dyn Fn() -> Result<String, Error>>) = (
                    filepath,
                    Box::new(move || {
                        // Replace the file prefix.
                        let re = Regex::new(r"^\(.+\)\s*").unwrap();
                        let result =
                            re.replace_all(filepath_rc.as_str(), env!("CARGO_MANIFEST_DIR"));

                        debug!("Script to be loaded into snapshot = {}", result);

                        Ok(fs::read_to_string(result.as_ref())?)
                    }),
                );

                js_files.push(pair);
            }

            extensions.push(Extension::builder().js(js_files).build())
        }

        RuntimeOptions {
            extensions,
            will_snapshot: true,
            ..Default::default()
        }
    }

    fn create_snapshot(runtime: &mut JsRuntime) -> Result<()> {
        let startup_data = runtime.snapshot();
        *SNAPSHOT.lock() = startup_data.to_vec();

        info!("Created a new snapshot");

        Ok(())
    }

    fn get_snapshot() -> Box<[u8]> {
        debug!("Using snapshot");
        // TODO: Explore Snapshot::Static(...) options.
        // Not going to be as fast as .as_ref() but it should suffice.
        SNAPSHOT.lock().clone().into_boxed_slice()
    }

    pub fn snapshot_exists() -> bool {
        // Caveat: Scripts loaded in snapshots are not watched for changes.
        if SNAPSHOT.lock().len() > 0 {
            return true;
        }

        false
    }

    fn handle_reciever_error<T: std::error::Error + 'static + Send + Sync>(
        result: std::result::Result<std::result::Result<(), deno_core::error::AnyError>, T>,
    ) -> Result<()> {
        result?.context("running the event loop".to_string())
    }
}

lazy_static! {
    static ref SNAPSHOT: Mutex<Vec<u8>> = Mutex::new(vec![]);
}
