// Copyright 2021 the Gigamono authors. All rights reserved. Apache 2.0 license.

// TODO(appcypher): Synchronisation needed with fcntl. Also applies to db.
// https://blog.cloudflare.com/durable-objects-easy-fast-correct-choose-three/
use std::cell::RefCell;
use std::rc::Rc;

use deno_core::{error::AnyError, include_js_files, op_async, Extension, OpState};

use crate::events::Events;
use crate::permissions::Permissions;

pub fn event_http(permissions: Rc<Permissions>, events: Rc<Events>) -> Extension {
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
            if !state.has::<Permissions>() {
                state.put(permissions.clone());
            }

            if !state.has::<Events>() {
                state.put(events.clone());
            }
            Ok(())
        })
        .build();

    extension
}

// TODO(appcypher)
async fn op_add_event_listener(_: Rc<RefCell<OpState>>, _: (), _: ()) -> Result<(), AnyError> {
    Ok(())
}
