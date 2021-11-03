extern crate secure_runtime;

use std::fs;

use secure_runtime::SecureRuntime;
use utilities::result::Result;

fn main() -> Result<()> {
    let permissions = Default::default();
    let mut runtime = SecureRuntime::new_default(permissions)?;

    let main_module_filename = "./examples/js/read_text_file.js";
    let main_module_code = fs::read_to_string(main_module_filename)?;

    runtime.execute_main_module(main_module_filename, main_module_code)
}
