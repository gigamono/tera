// Copyright 2021 the Gigamono authors. All rights reserved. Apache 2.0 license.
// Based on https://github.com/denoland/deno/tree/main/runtime. Copyright the Deno authors. MIT license.

"use strict";

((window) => {
  // const { Uint8Array } = window.__bootstrap.primordials;
  const { core } = window.__bootstrap;

  function sleep(text = "") {
    return core.encode(text);
  }

  window.__bootstrap.timers = { sleep };
})(globalThis);
