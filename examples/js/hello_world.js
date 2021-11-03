function greet(target) {
  return `hello ${target}`;
}

let greeting = greet("world");

sys.core.print(`${greeting}\n`);
