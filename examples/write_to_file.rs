extern crate secure_runtime;

use std::fs;

use secure_runtime::{
    permissions::{
        fs::{FSCapability, FS},
        Permissions,
    },
    set, SecureRuntime,
};
use utilities::result::Result;

fn main() -> Result<()> {
    // Create permitted resources
    let example_txt_dir = fs::canonicalize("./examples/txt")?.display().to_string();

    // Create permissions
    let permissions = Permissions::builder()
        .fs(FS::Open, set![example_txt_dir.clone()])
        .fs(FS::Create, set![example_txt_dir.clone()])
        .fs(FS::Write, set![example_txt_dir])
        .build();

    // Create a new runtime.
    let mut runtime = SecureRuntime::new_default(permissions)?;

    // Read main module code.
    let main_module_filename = "./examples/js/write_to_file.js";
    let main_module_code = fs::read_to_string(main_module_filename)?;

    // Execute main module.
    runtime.execute_main_module(main_module_filename, main_module_code)
}
