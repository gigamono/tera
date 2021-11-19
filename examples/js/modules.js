// Copyright 2021 the Gigamono authors. All rights reserved. Apache 2.0 license.

import { Rectangle } from "./shapes.js";

let rect = new Rectangle(5, 40);

sys.core.print(`area = ${rect.area()}\n`);
sys.core.print(`perimeter = ${rect.perimeter()}\n`);
