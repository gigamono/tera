// Copyright 2021 the Gigamono authors. All rights reserved. Apache 2.0 license.

extern crate tera;

use std::fs;

use tera::{permissions::Permissions, Runtime};
use utilities::result::Result;

#[tokio::main]
async fn main() -> Result<()> {
    // Create runtime with default permissions.
    let permissions = Permissions::default();
    let mut runtime = Runtime::default_main(permissions).await?;

    // Get main module code.
    let main_module_filename = "./examples/js/hello_world.js";
    let main_module_code = fs::read_to_string(main_module_filename)?;

    // Execute main module.
    runtime
        .execute_module(main_module_filename, main_module_code)
        .await
}
