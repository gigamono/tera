// Copyright 2021 the Gigamono authors. All rights reserved. Apache 2.0 license.

extern crate tera;

use std::{convert::TryFrom, fs};

use tera::{
    permissions::{
        fs::{Fs, FsPath, FsRoot},
        Permissions,
    },
    Runtime,
};
use utilities::result::Result;

#[tokio::main]
async fn main() -> Result<()> {
    // Create permitted resources
    let allow_list = [FsPath::from("/examples/txt/**")];

    // Create permissions
    let permissions = Permissions::builder()
        .add_state(FsRoot::try_from(env!("CARGO_MANIFEST_DIR"))?)
        .add_permissions_with_allow_lists(&[
            (Fs::Open, &allow_list),
            (Fs::Read, &allow_list),
            (Fs::Write, &allow_list),
        ])?
        .build();

    // Create a new runtime.
    let mut runtime = Runtime::with_permissions(permissions, false, Default::default()).await?;

    // Get the code.
    let code = fs::read_to_string("examples/js/files.js")?;

    fs::write("test.txt", "contents")?;

    // Execute main module.
    runtime.execute_module("/examples/js/files.js", code).await
}
