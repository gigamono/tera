// Copyright 2021 the Gigamono authors. All rights reserved. Apache 2.0 license.

extern crate tera;

use tera::{
    permissions::{
        fs::{PathString, FS},
        Permissions,
    },
    Runtime,
};
use tokio::fs;
use utilities::result::Result;

#[tokio::main]
async fn main() -> Result<()> {
    // Create permitted resources
    let allow_list = [PathString("./examples/txt".into())];

    // Create permissions
    let permissions = Permissions::builder()
        .add_permissions(&[(FS::Open, &allow_list), (FS::Read, &allow_list)])
        .build();

    // Create a new runtime.
    let mut runtime = Runtime::default_main(permissions).await?;

    // Read main module code.
    let main_module_filename = "./examples/js/read_text_file.js";
    let main_module_code = fs::read_to_string(main_module_filename).await?;

    // Execute main module.
    runtime
        .execute_module(main_module_filename, main_module_code)
        .await
}
