// Copyright 2021 the Gigamono authors. All rights reserved. Apache 2.0 license.
// TODO(appcypher): Synchronisation needed with fcntl. Also applies to db. https://blog.cloudflare.com/durable-objects-easy-fast-correct-choose-three/

use deno_core::error::type_error;
use deno_core::{
    error::AnyError, include_js_files, op_async, Extension, OpState, Resource, ResourceId,
};
use deno_core::{AsyncRefCell, RcRef, ZeroCopyBuf};
use serde::Deserialize;
use std::cell::RefCell;
use std::io::SeekFrom;
use std::rc::Rc;
use tokio::fs::{File, OpenOptions};
use tokio::io::{AsyncReadExt, AsyncSeekExt, AsyncWriteExt};

use crate::permissions::fs::{PathString, FS};
use crate::permissions::Permissions;

pub fn fs(permissions: Rc<Permissions>) -> Extension {
    let extension = Extension::builder()
        .js(include_js_files!(
            prefix "sys:ext/fs",
            "lib/extensions/fs/01_fs.js",
        ))
        .ops(vec![
            ("op_open", op_async(op_open)),
            ("op_fs_write", op_async(op_fs_write)),
            ("op_fs_read", op_async(op_fs_read)),
            ("op_fs_seek", op_async(op_fs_seek)),
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
    pub file: AsyncRefCell<File>,
    pub path: String,
    pub options: FileOptions,
}

#[derive(Deserialize, Default, Debug)]
pub struct FileOptions {
    pub write: bool,
    pub read: bool,
    pub append: bool,
    pub create: bool,
    pub truncate: bool,
}

impl Resource for FileResource {}

async fn op_open(
    state: Rc<RefCell<OpState>>,
    path: String,
    options: FileOptions,
) -> Result<ResourceId, AnyError> {
    let path = &path;

    // We use OS-supported permissions for files. Permissions are added on file open/creation.
    let permissions = state.borrow().borrow::<Rc<Permissions>>().clone();

    // Check create permission.
    if options.create {
        permissions
            .check(FS::Create, PathString(path.into()))
            .await?;
    } else {
        // Check open permission.
        permissions.check(FS::Open, PathString(path.into())).await?;
    }

    // Open file with options specified.
    let file = OpenOptions::new()
        .read(options.read)
        .write(options.write)
        .append(options.append)
        .truncate(options.truncate)
        .create(options.create)
        .open(path)
        .await?;

    // We move open, read, write permission checks here because if file does not exist yet, canonicalising won't work in permission checks.
    // Check read permission.
    if options.read {
        permissions.check(FS::Read, PathString(path.into())).await?;
    }

    // Check write permission.
    if options.write {
        permissions
            .check(FS::Write, PathString(path.into()))
            .await?;
    }

    // Save file info for later.
    let rid = state.borrow_mut().resource_table.add(FileResource {
        file: AsyncRefCell::new(file),
        path: path.into(),
        options, 
    });

    Ok(rid)
}

async fn op_fs_write(
    state: Rc<RefCell<OpState>>,
    rid: ResourceId,
    buf: ZeroCopyBuf,
) -> Result<usize, AnyError> {
    // SEC: No permission check because each file is opened with OS-supported perms.
    // TODO: Document. Also why unwrap?
    let resource = state
        .borrow()
        .resource_table
        .get::<FileResource>(rid)?
        .clone();

    let file_borrow = RcRef::map(&resource, |f| &f.file).try_borrow_mut();
    let mut file = file_borrow.unwrap().try_clone().await?;

    // Write to file.
    let total_written = file.write(&buf[..]).await?;

    // Flush to move intermediate buffered content to file.
    file.flush().await?;

    Ok(total_written)
}

async fn op_fs_read(
    state: Rc<RefCell<OpState>>,
    rid: ResourceId,
    mut buf: ZeroCopyBuf,
) -> Result<usize, AnyError> {
    // SEC: No permission check because each file is opened with OS-supported perms.
    // TODO: Document. Also why unwrap?
    let resource = state
        .borrow()
        .resource_table
        .get::<FileResource>(rid)?
        .clone();

    let file_borrow = RcRef::map(&resource, |f| &f.file).try_borrow_mut();
    let mut file = file_borrow.unwrap().try_clone().await?;

    // Read from file.
    let total_written = file.read(&mut buf[..]).await?;

    Ok(total_written)
}

#[derive(Deserialize, Debug)]
struct SeekArgs {
    offset: i64,
    whence: i32,
}

async fn op_fs_seek(
    state: Rc<RefCell<OpState>>,
    rid: ResourceId,
    args: SeekArgs,
) -> Result<u64, AnyError> {
    // SEC: No permission check because each file is opened with OS-supported perms.
    // TODO: Document. Also why unwrap?
    let resource = state
        .borrow()
        .resource_table
        .get::<FileResource>(rid)?
        .clone();

    let file_borrow = RcRef::map(&resource, |f| &f.file).try_borrow_mut();
    let mut file = file_borrow.unwrap().try_clone().await?;

    let seek = {
        match args.whence {
            0 => SeekFrom::Start(args.offset as u64),
            1 => SeekFrom::Start(args.offset as u64),
            2 => SeekFrom::Start(args.offset as u64),
            _ => {
                return Err(type_error(format!(
                    r#"invalid whence value "{}""#,
                    args.whence
                )))
            }
        }
    };

    // Seek file.
    let pos = file.seek(seek).await?;

    Ok(pos)
}
