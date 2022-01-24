// Copyright 2021 the Gigamono authors. All rights reserved. GPL-3.0 License.

import { Rectangle } from "./shapes.js";
const { log } = Tera;

async function main() {
  let rect = new Rectangle(5, 40);

  log.info("area =", rect.area());
  log.info("perimeter =", rect.perimeter());
}

if (import.meta.main) {
  await main();
}
