// Copyright 2021 the Gigamono authors. All rights reserved. Apache 2.0 license.

import { Rectangle } from "./shapes.js";

const { log } = Tera;

let rect = new Rectangle(5, 40);

log.info("area =", rect.area());
log.info("perimeter =", rect.perimeter());
