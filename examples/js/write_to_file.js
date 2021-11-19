// Copyright 2021 the Gigamono authors. All rights reserved. Apache 2.0 license.

try {
  const rid = await sys.open("examples/txt/write.txt", { write: true });
  await sys.writeAll(rid, "This is new content");
} catch (e) {
  sys.core.print(`error = ${e}\n`);
}
