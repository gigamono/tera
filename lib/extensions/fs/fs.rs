use std::cell::RefCell;
use std::rc::Rc;
use tokio::fs;

use deno_core::error::AnyError;
use deno_core::{include_js_files, op_async};
use deno_core::{Extension, OpState};

use crate::permissions::fs::FS;
use crate::permissions::Permissions;

pub fn fs(permissions: Rc<Permissions>) -> Extension {
    let extension = Extension::builder()
        .js(include_js_files!(
            prefix "sys:ext/fs",
            "lib/extensions/fs/01_files.js",
        ))
        .ops(vec![("op_read_text_file", op_async(op_read_text_file))])
        .build();

    // extension.init_state(OpState::new());
    extension
}

async fn op_read_text_file(
    state: Rc<RefCell<OpState>>,
    path: String,
    _: (),
) -> Result<String, AnyError> {
    // let op_state = state.borrow();

    // let permissions = op_state.borrow::<Rc<Permissions>>();
    // permissions.check(FS::Read, &path)?;

    let content = fs::read_to_string(path).await?;
    Ok(content)
}
