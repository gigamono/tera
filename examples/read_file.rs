extern crate secure_runtime;

use secure_runtime::{SecureRuntime, Source};

fn main() {
    let mut runtime = SecureRuntime::new();
    runtime.execute_module(&Source {
        filename: String::from("esm.js"),
        code: String::from(
            r#"
        const content = sys.core.opAsync("op_read");
        sys.core.print(`>> file content = "${content}"`);
        "#,
        ),
    });
}
