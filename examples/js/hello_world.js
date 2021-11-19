// Copyright 2021 the Gigamono authors. All rights reserved. Apache 2.0 license.

function greet(target) {
  return `hello ${target}`;
}

let greeting = greet("world");

sys.core.print(`${greeting}\n`);
