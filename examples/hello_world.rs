extern crate secure_runtime;

use secure_runtime::{SecureRuntime, Source};

fn main() {
    let mut runtime = SecureRuntime::new();
    runtime.execute_module(&Source {
        filename: String::from("esm.js"),
        code: String::from(
            r#"
        function greet(target) {
            return `hello ${target}`;
        }

        let greeting = greet("world");

        sys.core.print(`${greeting}\n`);
        "#,
        ),
    });
}
