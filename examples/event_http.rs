// Copyright 2021 the Gigamono authors. All rights reserved. Apache 2.0 license.

extern crate tera;

use tera::{permissions::Permissions, Runtime};
use tokio::fs;
use utilities::result::Result;

#[tokio::main]
async fn main() -> Result<()> {
    // Create permissions
    let permissions = Permissions::default();

    // Create a new runtime.
    let mut runtime = Runtime::default_event(permissions).await?;

    // Read main module code.
    let main_module_filename = "./examples/js/event_http.js";
    let main_module_code = fs::read_to_string(main_module_filename).await?;

    // Execute main module.
    runtime
        .execute_module(main_module_filename, main_module_code)
        .await
}
