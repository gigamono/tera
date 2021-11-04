extern crate secure_runtime;

use std::fs;

use secure_runtime::SecureRuntime;
use utilities::result::Result;

fn main() -> Result<()> {
    // Create runtime with default permissions.
    let permissions = Default::default();
    let mut runtime = SecureRuntime::new_default(permissions)?;

    // Get main module code.
    let main_module_filename = "./examples/js/hello_world.js";
    let main_module_code = fs::read_to_string(main_module_filename)?;

    // Execute main module.
    runtime.execute_main_module(main_module_filename, main_module_code)
}
