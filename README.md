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

There is plan to support WebAssembly in the future.

> If you are looking to get started with the Gigamono framework, check the [Gigamono repo](https://github.com/gigamono/gigamono).

##

### Content

1. [Usage](#usage)

##

### Usage <a name="usage" />

You need to add [tokio](https://crates.io/crates/tokio) to your dependencies.

<sup>
    <div align="center">
        :warning: The current API is subject to change. :warning:
    </div>
</sup>

```rs
#[tokio::main]
async fn main() -> Result<()> {
    // Create permitted resources
    let allow_list = [FsPath::from("/examples/txt/**")];

    // Create permissions
    let permissions = Permissions::builder()
        .add_state(FsRoot::try_from(env!("CARGO_MANIFEST_DIR"))?)
        .add_permissions_with_allow_lists(&[
            (Fs::Open, &allow_list),
            (Fs::Read, &allow_list),
            (Fs::Write, &allow_list),
        ])?
        .build();

    // Create a new runtime.
    let mut runtime =
        Runtime::with_permissions(permissions, false, vec![], Default::default()).await?;

    // Get the code.
    let code = fs::read_to_string("examples/js/files.js")?;

    // Execute main module.
    runtime.execute_module("/examples/js/files.js", code).await
}
```

Check the [examples folder](./examples) for more examples.
