extern crate secure_runtime;

use std::{fs, path::PathBuf};

use secure_runtime::{
    permissions::{
        fs::{FSCapability, FS},
        Permissions,
    },
    SecureRuntime,
};
use utilities::result::Result;

fn main() -> Result<()> {
    // Create permitted resources
    let example_js_dir = canonicalize("./examples/js")?.display().to_string();

    // Create permissions
    let permissions = Permissions::builder()
        .fs(FS::Read, vec![example_js_dir])
        .build();

    // Create a new runtime.
    let mut runtime = SecureRuntime::new_default(permissions)?;

    // Get module code.
    let main_module_filename = "./examples/js/modules.js";
    let main_module_code = fs::read_to_string(main_module_filename)?;

    // Execute main module.
    runtime.execute_main_module(main_module_filename, main_module_code)
}

fn canonicalize(path: &str) -> std::io::Result<PathBuf> {
    fs::canonicalize(path)
}
