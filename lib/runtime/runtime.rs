// Copyright 2021 the Gigamono authors. All rights reserved. Apache 2.0 license.

use crate::{events::Events, extensions, loaders, permissions::Permissions, RuntimeOptions};
use deno_core::{
    v8::{Global, Value},
    JsRuntime,
};
use log::debug;
use std::{cell::RefCell, path::PathBuf, rc::Rc};
use tokio::fs;
use utilities::result::{Context, Result};

pub struct Runtime {
    runtime: JsRuntime,
    permissions: Rc<RefCell<Permissions>>,
}

impl Runtime {
    pub async fn new(
        options: RuntimeOptions,
        permissions: Rc<RefCell<Permissions>>,
    ) -> Result<Self> {
        // TODO(appcypher): Add support for memory snapshot after initialisation that can then be reused each time.
        // TODO(appcypher): SEC: Support specifying maximum_heap_size_in_bytes.
        // TODO(appcypher): SEC: Add a callback that panics for near_heap_limit_callback.

        // Create a new runtime.
        let mut runtime = JsRuntime::new(options);

        // Execute postscripts.
        Self::execute_postscripts(&mut runtime).await?;

        debug!("Runtime started");

        Ok(Self {
            runtime,
            permissions,
        })
    }

    pub async fn with_permissions(
        permissions: Permissions,
        options: RuntimeOptions,
    ) -> Result<Self> {
        // TODO(appcypher): Support other options destructured to replace. ..Default::default()
        // TODO(appcypher): There should be a series of snapshots with different combination of extensions. Chosen based on permissions.
        let permissions = Rc::new(RefCell::new(permissions));

        // Set runtime options
        let opts = RuntimeOptions {
            module_loader: Some(Rc::new(loaders::esm(Rc::clone(&permissions)))),
            extensions: vec![extensions::fs(Rc::clone(&permissions))],
            ..options
        };

        Self::new(opts, permissions).await
    }

    pub async fn with_events(
        permissions: Permissions,
        events: Rc<RefCell<Events>>,
        options: RuntimeOptions,
    ) -> Result<Self> {
        // TODO(appcypher): Support other options destructured to replace. ..Default::default()
        // TODO(appcypher): There should be a series of snapshots with different combination of extensions. Chosen based on permissions.
        let permissions = Rc::new(RefCell::new(permissions));

        // Set runtime options
        let opts = RuntimeOptions {
            module_loader: Some(Rc::new(loaders::esm(Rc::clone(&permissions)))),
            extensions: vec![
                extensions::fs(Rc::clone(&permissions)),
                extensions::event_http(Rc::clone(&permissions), events),
            ],
            ..options
        };

        Self::new(opts, permissions).await
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

    async fn execute_postscripts(runtime: &mut JsRuntime) -> Result<()> {
        // TODO(appcypher): Instead of fetching the postscripts at runtime, we should add them statically at compile time. Embedded in the binary for faster load time. Minified maybe
        // TODO(appcypher): Also support skipping builtin postcripts and loading user-specified ones at runtime.
        // Get postcripts directory.
        let postscripts_dir =
            &PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("lib/runtime/postscripts");

        // Blindly assume everything in directory is a postscript.
        let mut postscripts = std::fs::read_dir(postscripts_dir)
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
                .await
                .context(format!(r#"getting postscript file, "{:?}""#, path))?;

            // Execute postscript.
            runtime
                .execute_script(&format!("(tera:postscripts) {}", &path.display()), &content)
                .context("executing postscript file")?;
        }

        Ok(())
    }

    fn handle_reciever_error<T: std::error::Error + 'static + Send + Sync>(
        result: std::result::Result<std::result::Result<(), deno_core::error::AnyError>, T>,
    ) -> Result<()> {
        result?.context("running the event loop".to_string())
    }
}
