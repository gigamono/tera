try {
  const rid = await sys.open("examples/txt/write.txt", { write: true });
  await sys.writeAll(rid, "This is new content");
} catch (e) {
  sys.core.print(`error = ${e}\n`);
}
