use std::cell::RefCell;
use std::fs;
use std::rc::Rc;

use deno_core::error::AnyError;
use deno_core::{include_js_files, op_async};
use deno_core::{Extension, OpState};

pub fn fs() -> Extension {
    Extension::builder()
        .js(include_js_files!(
            prefix "sys:ext/fs",
            "lib/extensions/fs/01_files.js",
        ))
        .ops(vec![
            ("op_read_text_file", op_async(op_read_text_file)),
        ])
        .build()
}

async fn op_read_text_file(
    _: Rc<RefCell<OpState>>,
    path: String,
    _: (),
) -> Result<String, AnyError> {
    let content = fs::read_to_string(path).unwrap(); // TODO(appcypher): Permissions. Limit Read. Use tokio
    Ok(content)
}
