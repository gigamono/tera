// Copyright 2021 the Gigamono authors. All rights reserved. Apache 2.0 license.

extern crate tera;

use tera::{
    permissions::{
        fs::{FilePathString, FS},
        Permissions,
    },
    Runtime,
};
use tokio::fs;
use utilities::result::Result;

#[tokio::main]
async fn main() -> Result<()> {
    // Create permitted resources
    let allow_list = [FilePathString("./examples/js".into())];

    // Create permissions
    let permissions = Permissions::builder()
        .add_permissions(&[(FS::Import, &allow_list)])
        .build();

    // Create a new runtime.
    let mut runtime = Runtime::default_main(permissions).await?;

    // Get main module code.
    let main_module_filename = "./examples/js/modules.js";
    let main_module_code = fs::read_to_string(main_module_filename).await?;

    // Execute main module.
    runtime
        .execute_module(main_module_filename, main_module_code)
        .await
}
