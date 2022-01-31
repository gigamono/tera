// Copyright 2021 the Gigamono authors. All rights reserved. GPL-3.0 License.
//! No support for non-ascii headers yet.

use crate::permissions::Permissions;
use deno_core::{error::AnyError, include_js_files, op_sync, Extension, OpState};
use std::cell::RefCell;
use std::rc::Rc;

pub fn env(permissions: Rc<RefCell<Permissions>>) -> Extension {
    let extension = Extension::builder()
        .js(include_js_files!(
            prefix "(tera:extensions) ",
            "lib/extensions/env/01_env.js",
        ))
        .ops(vec![("opEnvGet", op_sync(op_env_get))])
        .state(move |state| {
            if !state.has::<Rc<RefCell<Permissions>>>() {
                state.put(Rc::clone(&permissions));
            }

            Ok(())
        })
        .build();

    extension
}

fn op_env_get(_: &mut OpState, _: (), _: ()) -> Result<String, AnyError> {
    // TODO(appcypher): Add implementation
    Ok(String::new())
}
