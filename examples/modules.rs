// Copyright 2021 the Gigamono authors. All rights reserved. GPL-3.0 License.

extern crate tera;

use std::convert::TryFrom;

use tera::{
    permissions::{
        fs::{Fs, FsPath, FsRoot},
        Permissions,
    },
    Runtime,
};
use tokio::fs;
use utilities::result::Result;

#[tokio::main]
async fn main() -> Result<()> {
    // Create permitted resources
    let allow_list = [FsPath::from("/examples/js/**")];

    // Create permissions
    let permissions = Permissions::builder()
        .add_state(FsRoot::try_from(env!("CARGO_MANIFEST_DIR"))?)
        .add_permissions_with_allow_lists(&[(Fs::Execute, &allow_list)])?
        .build();

    // Create a new runtime.
    let mut runtime =
        Runtime::with_permissions(permissions, false, vec![], Default::default()).await?;

    // Get main module code.
    let main_module_code = fs::read_to_string("examples/js/modules.js").await?;

    // Execute main module.
    runtime
        .execute_module("/examples/js/modules.js", main_module_code)
        .await
}
