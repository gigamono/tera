//! To be removed. For development only.

use std::cell::RefCell;
use std::rc::Rc;

use deno_core::error::AnyError;
use deno_core::{include_js_files, op_async, op_sync};
use deno_core::{Extension, OpState, ResourceId};

pub fn dev() -> Extension {
    Extension::builder()
        .js(include_js_files!(
            prefix "sys:ext/fs",
            "lib/extensions/dev/01_dev.js",
        ))
        .ops(vec![
            ("op_dev_async", op_async(op_dev_async)),
            ("op_dev_sync", op_sync(op_dev_sync)),
        ])
        .build()
}

async fn op_dev_async(
    _state: Rc<RefCell<OpState>>,
    _conn_rid: ResourceId,
    _: (),
) -> Result<String, AnyError> {
    Ok(String::from("content from op_dev_ASYNC"))
}

fn op_dev_sync(_state: &mut OpState, _conn_rid: ResourceId, _: ()) -> Result<String, AnyError> {
    Ok(String::from("content from op_dev_SYNC"))
}
