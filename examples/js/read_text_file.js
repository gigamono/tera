try {
  const content = await sys.readTextFile("examples/txt/lorem.txt");
  sys.core.print(`>> file content = "${content}"\n`);
} catch (e) {
  sys.core.print(`error = ${e}\n`);
}
