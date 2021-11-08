extern crate secure_runtime;

use std::fs;

use secure_runtime::{SecureRuntime, permissions::{Permissions, fs::{FS, FSCapability}}, set};
use utilities::result::Result;

#[tokio::main]
async fn main() -> Result<()> {
    // Create permitted resources
    let example_txt_dir = fs::canonicalize("./examples/txt")?.display().to_string();

    // Create permissions
    let permissions = Permissions::builder()
        .fs(FS::Read, set![example_txt_dir])
        .build();

    // Create a new runtime.
    let mut runtime = SecureRuntime::new_default(permissions).await?;

    // Read main module code.
    let main_module_filename = "./examples/js/read_text_file.js";
    let main_module_code = fs::read_to_string(main_module_filename)?;

    // Execute main module.
    runtime.execute_main_module(main_module_filename, main_module_code).await
}
