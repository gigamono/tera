// Copyright 2021 the Gigamono authors. All rights reserved. Apache 2.0 license.

extern crate tera;

use tera::{
    permissions::{
        fs::{Path, Root, FS},
        Permissions,
    },
    Runtime,
};
use utilities::result::Result;

#[tokio::main]
async fn main() -> Result<()> {
    // Create permitted resources
    let allow_list = [Path::from(concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/",
        "examples/txt"
    ))];

    // Create permissions
    let permissions = Permissions::builder()
        .add_state(Root::from(env!("CARGO_MANIFEST_DIR")))
        .add_permissions(&[(FS::Open, &allow_list), (FS::Read, &allow_list)])?
        .build();

    // Create a new runtime.
    let mut runtime = Runtime::default_main(permissions).await?;

    // Execute main module.
    runtime
        .execute_module(
            "examples/js/read_text_file.js",
            r#"
            const file = await File.open("examples/txt/lorem.txt", { read: true });
            const buf = await file.readAll();
            log.info(">> file content =", decode(buf));
          "#,
        )
        .await
}
