<div align="center">
    <a href="#" target="_blank">
        <img src="https://raw.githubusercontent.com/appcypher/gigamono-assets/main/avatar-gigamono-boxed.png" alt="Gigamono Logo" width="140" height="140"></img>
    </a>
</div>

<h1 align="center">Tera</h1>

`tera` is a lean secure capability-based runtime for JavaScript. It is primarily designed for multi-tenant serverless environment but has uses in other contexts.

`tera` provides a small set of low-level permission-enabled APIs that you can get started with but it can also be extended with your own permission types and host API implementation.

There is currently no plan to strictly support web-compatible APIs or out-of-the-box typescript compilation. If need those functionalities, take a look at [deno](https://github.com/denoland/deno).

`tera` is based on [deno core](https://github.com/denoland/deno/tree/main/core) and inspired by [deno runtime](https://github.com/denoland/deno/tree/main/runtime).

There is plan to support WebAssembly with zero cold start in the future.

> Information provided here is for folks working on this package. If your goal is to get started with the Gigamono framework, check the [Gigamono repo](https://github.com/gigamono/gigamono) on how to do that.

##

### Content

1. [Usage](#usage)

##

### Usage <a name="usage" />

You need to add [tokio](https://crates.io/crates/tokio) to your dependencies.

:warning: The current API is subject to change.

```rs
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
    // Create allowed resources
    let allow_list = [PathString("./examples/txt".into())];

    // Create permissions object.
    let permissions = Permissions::builder()
        .add_permissions(&[
            (FS::Open, &allow_list),
            (FS::Read, &allow_list)
        ])
        .build();

    // Start a new js runtime.
    let mut runtime = Runtime::default_main(permissions).await?;

    // Get code and main module filename.
    let main_module_filename = "./examples/js/read_text_file.js";
    let main_module_code = fs::read_to_string(main_module_filename).await?;

    // Execute the main module code.
    runtime
        .execute_module(main_module_filename, main_module_code)
        .await
}
```
