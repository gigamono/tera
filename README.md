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

:warning: The current API is subject to change.

```rs
#[tokio::main]
async fn main() -> Result<()> {
    // Create permitted resources
    let allow_list = [Path::from(concat!(env!("CARGO_MANIFEST_DIR"), "/", "examples/txt"))];

    // Create permissions
    let permissions = Permissions::builder()
        .add_state(Root::from(env!("CARGO_MANIFEST_DIR"))) // Root folder for files to be accessed.
        .add_permissions(&[
            (FS::Open, &allow_list),
            (FS::Read, &allow_list),
        ])?
        .build();

    // Create a new runtime.
    let mut runtime = Runtime::default_main(permissions).await?;

    // Execute main module.
    runtime
        .execute_module(
            "examples/read_text_file.js",
            r#"
            const readFile = await File.open("examples/txt/lorem.txt", { read: true });
            const readBuf = await readFile.readAll();
            log.info(">> file content =", decode(readBuf));
          "#,
        )
        .await
}
```

Check the [examples folder](./examples) for more examples.
