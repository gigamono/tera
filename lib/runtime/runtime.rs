// Copyright 2021 the Gigamono authors. All rights reserved. Apache 2.0 license.

use crate::{events::Events, extensions, loaders, permissions::Permissions, RuntimeOptions};
use deno_core::JsRuntime;
use log::debug;
use std::{cell::RefCell, path::PathBuf, rc::Rc};
use tokio::fs;
use utilities::result::{Context, Result};

pub struct Runtime(JsRuntime);

impl Runtime {
    async fn new(options: RuntimeOptions) -> Result<Self> {
        // TODO(appcypher): Add support for memory snapshot after initialisation that can then be reused each time.
        // TODO(appcypher): SEC: Support specifying maximum_heap_size_in_bytes.
        // TODO(appcypher): SEC: Add a callback that panics for near_heap_limit_callback.

        // Create a new runtime.
        let mut runtime = JsRuntime::new(options);

        // Execute postscripts.
        Self::execute_postscripts(&mut runtime).await?;

        debug!("Runtime started");

        Ok(Self(runtime))
    }

    pub async fn default_main(permissions: Permissions) -> Result<Self> {
        // TODO(appcypher): There should be a series of snapshots with different combination of extensions. Chosen based on permissions.
        let permissions = Rc::new(permissions);

        // Set runtime options
        let opts = RuntimeOptions {
            module_loader: Some(Rc::new(loaders::esm(permissions.clone()))),
            extensions: vec![extensions::fs(permissions.clone())],
            ..Default::default()
        };

        Self::new(opts).await
    }

    pub async fn default_event(permissions: Permissions, events: Rc<RefCell<Events>>) -> Result<Self> {
        // TODO(appcypher): There should be a series of snapshots with different combination of extensions. Chosen based on permissions.
        let permissions = Rc::new(permissions);

        // Set runtime options
        let opts = RuntimeOptions {
            module_loader: Some(Rc::new(loaders::esm(permissions.clone()))),
            extensions: vec![
                extensions::fs(permissions.clone()),
                extensions::event_http(permissions.clone(), events),
            ],
            ..Default::default()
        };

        Self::new(opts).await
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
            .0
            .load_main_module(&module_specifier, Some(module_code))
            .await
            .context("loading the main module")?;

        // Run main module.
        let mut rx = self.0.mod_evaluate(module_id);

        // Wait for message from module eval or event loop.
        // TODO(appcypher): Test that unawaited async ops are not getting executed in js. open, fs_write for example.
        tokio::select! {
            maybe_result = &mut rx => {
                Self::handle_reciever_error(maybe_result)?;

                // Continue event loop.
                self.0.run_event_loop(false).await.context("running the event loop")?;
            }
            event_loop_result = self.0.run_event_loop(false) => {
                event_loop_result.context("running the event loop")?;

                // Continue waiting on reciever.
                Self::handle_reciever_error(rx.await)?;
            }
        };

        Ok(())
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
                .execute_script(&format!("sys:ext/{:?}", &path), &content)
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
