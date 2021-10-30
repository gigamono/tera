use std::rc::Rc;
// use utilities::result::Result;
use crate::{extensions, loaders};
use deno_core::{JsRuntime, RuntimeOptions};

pub struct SecureRuntime(JsRuntime);

pub struct Source {
    pub filename: String,
    pub code: String,
}

impl SecureRuntime {
    pub fn new() -> Self {
        // TODO(appcypher): Snapshot right after JsRuntime::new. And add snapshot to options on start
        // Create a new runtime.
        let mut runtime = JsRuntime::new(RuntimeOptions {
            module_loader: Some(Rc::new(loaders::dev())),
            extensions: vec![extensions::fs()],
            ..Default::default()
        });

        // Execute post_script. It sets the "sys" namespace among other things.
        let post_script = Self::get_post_script();
        runtime
            .execute_script(&post_script.filename, &post_script.code)
            .unwrap(); // TODO(appcypher)

        Self(runtime)
    }

    pub fn execute_module(&mut self, src: &Source) {
        // TODO(appcypher) Run module.
        let tokio_runtime = tokio::runtime::Builder::new_current_thread()
            .enable_all()
            .build()
            .unwrap(); // TODO(appcypher)

        // TODO(appcypher)
        let future = async {
            self.0.execute_script(&src.filename, &src.code).unwrap();
            self.0.run_event_loop(false).await
        };

        // TODO(appcypher)
        tokio_runtime.block_on(future).unwrap();
    }

    fn get_post_script() -> Source {
        let rel_path = "lib/runtime/post_script.js";
        let path = format!("{}/{}", env!("CARGO_MANIFEST_DIR"), rel_path);
        let code = std::fs::read_to_string(path).unwrap(); // TODO(appcypher): Also use await
        Source {
            filename: format!("sys:ext/{}", rel_path),
            code: code,
        }
    }
}

mod test {
    use super::{SecureRuntime, Source};

    #[test]
    fn check_only_sys_namespace_visible() {
        let mut runtime = SecureRuntime::new();

        runtime.execute_module(&Source {
            filename: String::from("esm.js"),
            code: String::from("__bootstrap"),
        });

        runtime.execute_module(&Source {
            filename: String::from("esm.js"),
            code: String::from("Deno"),
        });
    }

    #[test]
    fn check_unable_to_set_object_proto() {
        let mut runtime = SecureRuntime::new();

        runtime.execute_module(&Source {
            filename: String::from("esm.js"),
            code: String::from("sys.__proto__ = {}"),
        });

        runtime.execute_module(&Source {
            filename: String::from("esm.js"),
            code: String::from(
                r#"
            let test = { __proto__: "gibberish" };
            test.__proto__;
            "#,
            ),
        });
    }

    #[test]
    fn check_sys_object_non_writable() {
        let mut runtime = SecureRuntime::new();

        runtime.execute_module(&Source {
            filename: String::from("esm.js"),
            code: String::from("sys = {}"),
        });
    }
}
