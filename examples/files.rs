// Copyright 2021 the Gigamono authors. All rights reserved. Apache 2.0 license.

extern crate tera;

use tera::{
    permissions::{
        fs::{Fs, FsPath, FsRoot},
        Permissions,
    },
    Runtime,
};
use utilities::result::Result;

#[tokio::main]
async fn main() -> Result<()> {
    // Create permitted resources
    let allow_list = [FsPath::from(concat!(
        env!("CARGO_MANIFEST_DIR"),
        "/",
        "examples/txt"
    ))];

    // Create permissions
    let permissions = Permissions::builder()
        .add_state(FsRoot::from(env!("CARGO_MANIFEST_DIR")))
        .add_permissions(&[
            (Fs::Open, &allow_list),
            (Fs::Read, &allow_list),
            (Fs::Write, &allow_list),
        ])?
        .build();

    // Create a new runtime.
    let mut runtime = Runtime::default_main(permissions).await?;

    // Execute main module.
    runtime
        .execute_module(
            "examples/js/read_text_file.js",
            r#"
            const readFile = await File.open("examples/txt/lorem.txt", { read: true });
            const readBuf = await readFile.readAll();
            log.info(">> file content =", decode(readBuf));

            const writeFile = await File.open("examples/txt/write.txt", { write: true });
            const writeString = `This is a random value written to a file: ${Math.random()}`;
            log.info(">> write string =", writeString);
            await writeFile.writeAll(encode(writeString));
          "#,
        )
        .await
}
