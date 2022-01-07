// Copyright 2021 the Gigamono authors. All rights reserved. Apache 2.0 license.

const { File, log, decode, encode } = Tera;

async function main() {
  // Read and write to the same file.
  const file = await File.open("/examples/txt/files.txt", {
    read: true,
    write: true,
  });

  const content = await file.readAll();

  log.info(">> file content =", decode(content));

  const writeContent = `This is a random value from Tera Js: ${Math.random()}\n`;

  await file.writeAll(encode(writeContent));
}

if (import.meta.main) {
  await main();
}
