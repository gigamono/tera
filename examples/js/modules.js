import { Rectangle } from "./shapes.js";

// let rect = Rectangle(5, 40);
let rect = new Rectangle(5, 40);

sys.core.print(`area = ${rect.area()}\n`);
sys.core.print(`perimeter = ${rect.perimeter()}\n`);
