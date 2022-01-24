// Copyright 2021 the Gigamono authors. All rights reserved. GPL-3.0 License.

// The Rectangle class.
export class Rectangle {
  constructor(height, width) {
    this.height = height;
    this.width = width;
  }

  area() {
    return this.height * this.width;
  }

  perimeter() {
    return 2 * (this.height + this.width);
  }
}

// The Square class.
export class Square {
  constructor(length) {
    this.length = length;
  }

  area() {
    return this.length * this.length;
  }

  perimeter() {
    return 4 * this.length;
  }
}

// The Circle class.
export class Circle {
  constructor(radius) {
    this.radius = radius;
  }

  area() {
    return Math.PI * Math.pow(this.radius, 2);
  }

  perimeter() {
    return 2 * Math.PI * this.radius;
  }
}
