// Copyright 2021 the Gigamono authors. All rights reserved. Apache 2.0 license.

// TODO(appcypher): Synchronisation needed with fcntl. Also applies to db.
// https://blog.cloudflare.com/durable-objects-easy-fast-correct-choose-three/
use serde::Deserialize;
use std::cell::RefCell;
use std::rc::Rc;
use tokio::fs::{self, File, OpenOptions};
use tokio::io::AsyncWriteExt;
use utilities::errors;

use deno_core::{
    error::AnyError, include_js_files, op_async, Extension, OpState, Resource, ResourceId,
};

use crate::permissions::Permissions;
use utilities::events::HttpEvent;

pub fn event_http(permissions: Rc<Permissions>, event: HttpEvent) -> Extension {
    let extension = Extension::builder()
        .js(include_js_files!(
            prefix "sys:ext/fs",
            "lib/extensions/fs/01_event_fetch.js",
        ))
        .ops(vec![(
            "op_add_event_listener",
            op_async(op_add_event_listener),
        )])
        .state(move |state| {
            state.put(permissions.clone());
            Ok(())
        })
        .build();

    extension
}

// TODO(appcypher)
async fn op_add_event_listener(_: Rc<RefCell<OpState>>, _: (), _: ()) -> Result<(), AnyError> {
    Ok(())
}
