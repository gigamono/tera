use deno_core::error::Context;
use serde::Deserialize;
use std::cell::RefCell;
use std::rc::Rc;
use tokio::fs::{self, File, OpenOptions};
use tokio::io::AsyncWriteExt;
use utilities::errors;

use deno_core::{
    error::AnyError, include_js_files, op_async, Extension, OpState, Resource, ResourceId,
};

use crate::permissions::fs::FS;
use crate::permissions::Permissions;

pub fn fs(permissions: Rc<Permissions>) -> Extension {
    let extension = Extension::builder()
        .js(include_js_files!(
            prefix "sys:ext/fs",
            "lib/extensions/fs/01_files.js",
        ))
        .ops(vec![
            ("op_read_text_file", op_async(op_read_text_file)),
            ("op_open", op_async(op_open)),
            ("op_write_all", op_async(op_write_all)),
        ])
        .state(move |state| {
            state.put(permissions.clone());
            Ok(())
        })
        .build();

    extension
}

#[derive(Debug)]
pub struct FileResource {
    pub file: RefCell<File>,
    pub path: String,
    pub options: FileOptions,
}

#[derive(Deserialize, Default, Debug)]
pub struct FileOptions {
    pub write: bool,
    pub read: bool,
    pub append: bool,
    pub truncate: bool,
}

impl Resource for FileResource {}

async fn op_open(
    state: Rc<RefCell<OpState>>,
    path: String,
    options: FileOptions,
) -> Result<ResourceId, AnyError> {
    // TODO(appcypher): What happens if one runtime opens a file in read-only mode and another runtime in the process opens it in write mode and tries to write to it.
    // Check open permissions.
    state
        .borrow()
        .borrow::<Rc<Permissions>>()
        .check(FS::Open, &path)?;

    // Open file with options specified.
    let file = OpenOptions::new()
        .read(options.read)
        .write(options.write)
        .append(options.append)
        .truncate(options.truncate)
        .open(&path)
        .await
        .context(format!(r#"opening file "{}""#, &path))?;

    // Save file info for later.
    let rid = state.try_borrow_mut()?.resource_table.add(FileResource {
        file: RefCell::new(file),
        path,
        options,
    });

    Ok(rid)
}

async fn op_read_text_file(
    state: Rc<RefCell<OpState>>,
    path: String,
    _: (),
) -> Result<String, AnyError> {
    // Check open permission.
    state
        .borrow()
        .borrow::<Rc<Permissions>>()
        .check(FS::Open, &path)?;

    // Check read permission.
    state
        .borrow()
        .borrow::<Rc<Permissions>>()
        .check(FS::Read, &path)?;

    // Read file content.
    let content = fs::read_to_string(&path)
        .await
        .context(format!(r#"reading file content "{}""#, &path))?;

    Ok(content)
}

async fn op_write_all(
    state: Rc<RefCell<OpState>>,
    rid: ResourceId,
    content: String,
) -> Result<(), AnyError> {
    // Get file resource.
    let res = state.borrow().resource_table.get::<FileResource>(rid)?;

    // Check if file has a write option.
    // TODO(appcypher): This may not be needed if the question in `op_open` is answered.
    if !res.options.write {
        errors::permission_error("file does not have write mode")?
    }

    // Check permissions.
    state
        .borrow()
        .borrow::<Rc<Permissions>>()
        .check(FS::Write, &res.path)?;

    // Write to file.
    res.file
        .try_borrow_mut()?
        .write_all(content.as_bytes())
        .await
        .context(format!(r#"writing content to file "{}""#, &res.path))?;

    Ok(())
}
