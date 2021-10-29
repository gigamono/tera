use std::cell::RefCell;
use std::rc::Rc;

use deno_core::error::AnyError;
use deno_core::{include_js_files, op_async, op_sync};
use deno_core::{Extension, OpState, ResourceId};

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
    _: ResourceId,
    _: (),
) -> Result<String, AnyError> {
    Ok(String::from("content from not-a-file"))
}
