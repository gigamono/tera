// Copyright 2021 the Gigamono authors. All rights reserved. GPL-3.0 License.
// Based on https://github.com/denoland/deno/tree/main/runtime. Copyright the Deno authors. MIT license.

"use strict";

((window) => {
  const { AssertionError } = window.__bootstrap.errors;

  function assert(cond, msg = "Assertion failed.") {
    if (!cond) {
      throw new AssertionError(msg);
    }
  }

  const SIZE_PER_ITER = 16 * 1024; // 16kb, see https://github.com/denoland/deno/issues/10157

  window.__bootstrap.common = { assert, SIZE_PER_ITER };
})(globalThis);
