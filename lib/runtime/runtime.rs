use crate::{extensions, loaders, permissions::Permissions};
use deno_core::{self, Extension, JsRuntime, ModuleLoader, RuntimeOptions};
use std::{path::PathBuf, rc::Rc};
use utilities::{
    errors::Error,
    result::{Context, Result},
};
pub struct SecureRuntime(JsRuntime);

pub struct Source {
    pub filename: String,
    pub code: String,
}

impl SecureRuntime {
    pub fn new(module_loader: Rc<dyn ModuleLoader>, extensions: Vec<Extension>) -> Result<Self> {
        // TODO(appcypher):
        //  Add support for snapshot.

        // Create a new runtime.
        let mut runtime = JsRuntime::new(RuntimeOptions {
            module_loader: Some(module_loader),
            extensions,
            ..Default::default()
        });

        // Execute post_script. It sets the "sys" namespace among other things.
        let post_script = Self::get_post_script()?;
        runtime
            .execute_script(&post_script.filename, &post_script.code)
            .context("executing post script")?;

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
        let module_specifier =
            deno_core::resolve_url(&format!("file://{}", filename)).context(format!(
                r#"resolving main module specifier as "file://{}""#,
                filename
            ))?;

        let tokio_runtime = tokio::runtime::Builder::new_current_thread()
            .enable_all()
            .build()
            .context("creating a tokio runtime")?;

        let evaluate_fut = async {
            let module_id = self
                .0
                .load_main_module(&module_specifier, Some(module_code))
                .await
                .context("loading the main module")?;

            let mut rx = self.0.mod_evaluate(module_id);

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

        tokio_runtime.block_on(evaluate_fut)?;

        Ok(())
    }

    fn get_post_script() -> Result<Source> {
        // TODO(appcypher): Instead of getting the post_sript at runtime, we should add statically at compile time. Maybe as a snapshot.
        let rel_path = "lib/runtime/post_script.js";
        let path = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join(rel_path);
        let code = std::fs::read_to_string(&path)
            .context(format!(r#"getting post_script file, "{:?}""#, path))?;

        Ok(Source {
            filename: format!("sys:ext/{:?}", rel_path),
            code: code,
        })
    }

    fn handle_reciever_error<T: std::error::Error + 'static>(
        result: std::result::Result<std::result::Result<(), deno_core::error::AnyError>, T>,
    ) -> Result<()> {
        // TODO
        let val = result.unwrap();

        val.context("running the event loop".to_string())
    }
}
