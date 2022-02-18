// Copyright 2021 the Gigamono authors. All rights reserved. GPL-3.0 License.
//! No support for non-ascii headers yet.

use crate::permissions::Permissions;
use deno_core::{error::AnyError, include_js_files, op_sync, Extension, OpState};
use std::cell::RefCell;
use std::rc::Rc;

pub fn cache(permissions: Rc<RefCell<Permissions>>) -> Extension {
    let extension = Extension::builder()
        .js(include_js_files!(
            prefix "(tera:extensions) ",
            "lib/extensions/cache/01_cache.js",
        ))
        .ops(vec![("opCacheGet", op_sync(op_cache_get))])
        .state(move |state| {
            if !state.has::<Rc<RefCell<Permissions>>>() {
                state.put(Rc::clone(&permissions));
            }

            Ok(())
        })
        .build();

    extension
}

// TODO(appcypher): Uses rocksdb to persist. Keeps values in memory. An Arc<Vec<u8>> is passed from server here. Flushes writes.

fn op_cache_get(_: &mut OpState, _: (), _: ()) -> Result<String, AnyError> {
    // TODO(appcypher): Add implementation
    Ok(String::new())
}
