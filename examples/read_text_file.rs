extern crate secure_runtime;

use secure_runtime::{SecureRuntime, Source};

fn main() {
    let mut runtime = SecureRuntime::new();
    runtime.execute_module(&Source {
        filename: String::from("esm.js"),
        code: String::from(
            r#"
        (async () => {
            try {
                const content = await sys.readTextFile("examples/lorem.txt");
                sys.core.print(`>> file content = "${content}"\n`);
            } catch (e) {
                sys.core.print(`error = ${e}\n`);
            }
        })();
        "#,
        ),
    });
}
