use crate::{extensions, loaders, permissions::Permissions};
use deno_core::{self, Extension, JsRuntime, ModuleLoader, RuntimeOptions};
use std::{path::PathBuf, rc::Rc};
use tokio::fs;
use utilities::{
    errors::{Error, SystemError},
    result::{Context, Result},
};
pub struct SecureRuntime(JsRuntime);

pub struct Source {
    pub filename: String,
    pub code: String,
}

impl SecureRuntime {
    pub fn new(module_loader: Rc<dyn ModuleLoader>, extensions: Vec<Extension>) -> Result<Self> {
        // TODO(appcypher): Add support for snapshot.

        // Create a new runtime.
        let mut runtime = JsRuntime::new(RuntimeOptions {
            module_loader: Some(module_loader),
            extensions,
            ..Default::default()
        });

        // Execute postscripts.
        Self::execute_postscripts(&mut runtime)?;

        Ok(Self(runtime))
    }

    pub fn new_default(permissions: Permissions) -> Result<Self> {
        // Wrap permissions in smart ptr to be potentially used between threads.
        let permissions = Rc::new(permissions);

        // Create default module loader and extensions.
        let module_loader = Rc::new(loaders::esm(permissions.clone()));
        let extensions = vec![extensions::fs(permissions.clone())];

        Self::new(module_loader, extensions)
    }

    pub fn execute_main_module(&mut self, filename: &str, module_code: String) -> Result<()> {
        // Add file scheme to filename and resolve to URL.
        let module_specifier =
            deno_core::resolve_url(&format!("file://{}", filename)).context(format!(
                r#"resolving main module specifier as "file://{}""#,
                filename
            ))?;

        // Create a runtime to run on current thread.
        let tokio_runtime = tokio::runtime::Builder::new_current_thread()
            .enable_all()
            .build()
            .context("creating a tokio runtime")?;

        let evaluate_fut = async {
            // Load main module and deps.
            let module_id = self
                .0
                .load_main_module(&module_specifier, Some(module_code))
                .await
                .context("loading the main module")?;

            // Run main module.
            let mut rx = self.0.mod_evaluate(module_id);

            // Wait for message from module eval or event loop.
            tokio::select! {
                cancellable = &mut rx => {
                    Self::handle_reciever_error(cancellable)?;
                }
                result = self.0.run_event_loop(false) => {
                    result.context("running the event loop")?;
                    Self::handle_reciever_error(rx.await)?;
                }
            };

            Ok::<(), Error>(())
        };

        tokio_runtime.block_on(evaluate_fut)
    }

    fn execute_postscripts(runtime: &mut JsRuntime) -> Result<()> {
        // TODO(appcypher): Instead of getting the postscripts at runtime, we should add them statically at compile time. Maybe as a snapshot.
        // Create a runtime to run on current thread.
        let tokio_runtime = tokio::runtime::Builder::new_current_thread()
            .enable_all()
            .build()
            .context("creating a tokio runtime")?;

        let read_file_fut = async {
            // Get postcripts directory.
            let postscripts_dir =
                &PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("lib/runtime/postscripts");

            // SEC: Blindly assume everything in directory is a postscript.
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

            Ok::<(), SystemError>(())
        };

        tokio_runtime.block_on(read_file_fut)
    }

    fn handle_reciever_error<T: std::error::Error + 'static + Send + Sync>(
        result: std::result::Result<std::result::Result<(), deno_core::error::AnyError>, T>,
    ) -> Result<()> {
        result?.context("running the event loop".to_string())
    }
}
