// Copyright 2021 the Gigamono authors. All rights reserved. Apache 2.0 license.
// TODO(appcypher): Synchronisation needed with fcntl. Also applies to db. https://blog.cloudflare.com/durable-objects-easy-fast-correct-choose-three/

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
use utilities::errors;

use crate::permissions::fs::{Path, FS};
use crate::permissions::Permissions;

pub fn fs(permissions: Rc<Permissions>) -> Extension {
    let extension = Extension::builder()
        .js(include_js_files!(
            prefix "(tera:extensions) ",
            "lib/extensions/fs/01_fs.js",
        ))
        .ops(vec![
            ("opFsOpen", op_async(op_fs_open)),
            ("opFsWrite", op_async(op_fs_write)),
            ("opFsRead", op_async(op_fs_read)),
            ("opFsSeek", op_async(op_fs_seek)),
        ])
        .state(move |state| {
            if !state.has::<Permissions>() {
                state.put(Rc::clone(&permissions));
            }

            Ok(())
        })
        .build();

    extension
}

#[derive(Debug)]
struct FileResource {
    file: AsyncRefCell<File>,
    path: String,
    options: FileOptions,
}

#[derive(Deserialize, Default, Debug)]
struct FileOptions {
    write: bool,
    read: bool,
    append: bool,
    create: bool,
    truncate: bool,
}

impl Resource for FileResource {}

async fn op_fs_open(
    state: Rc<RefCell<OpState>>,
    path: String,
    options: FileOptions,
) -> Result<ResourceId, AnyError> {
    // TODO(appcypher): SEC: Support root prefix for path.
    let path = &path;

    // We use OS-supported permissions for files. Permissions are added on file open/creation.
    let permissions = Rc::clone(state.borrow().borrow::<Rc<Permissions>>());

    // Check create permission.
    if options.create {
        permissions.check(FS::Create, Path::from(path))?;
    } else {
        // Check open permission.
        permissions.check(FS::Open, Path::from(path))?;
    }

    // Check read permission.
    if options.read {
        permissions.check(FS::Read, Path::from(path))?;
    }

    // Check write permission.
    if options.write {
        permissions.check(FS::Write, Path::from(path))?;
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
    let resource = state.borrow().resource_table.get::<FileResource>(rid)?;

    let mut file_rc = RcRef::map(&resource, |f| &f.file).borrow_mut().await;
    let file = file_rc.as_mut();

    // Write to file.
    let total_written = file.write(&buf).await?;

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
    let resource = state.borrow().resource_table.get::<FileResource>(rid)?;

    let mut file_rc = RcRef::map(&resource, |f| &f.file).borrow_mut().await;
    let file = file_rc.as_mut();

    // Read from file.
    let total_read = file.read(&mut buf).await?;

    Ok(total_read)
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
    let resource = state.borrow().resource_table.get::<FileResource>(rid)?;

    let mut file_rc = RcRef::map(&resource, |f| &f.file).borrow_mut().await;
    let file = file_rc.as_mut();

    let seek = {
        match args.whence {
            0 => SeekFrom::Start(args.offset as u64),
            1 => SeekFrom::Start(args.offset as u64),
            2 => SeekFrom::Start(args.offset as u64),
            _ => return errors::type_error_t(format!(r#"invalid whence value "{}""#, args.whence)),
        }
    };

    // Seek file.
    let pos = file.seek(seek).await?;

    Ok(pos)
}
