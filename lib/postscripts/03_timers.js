// Copyright 2021 the Gigamono authors. All rights reserved. GPL-3.0 License.
// Based on https://github.com/denoland/deno/tree/main/runtime. Copyright the Deno authors. MIT license.

"use strict";

((window) => {
  // const { Uint8Array } = window.__bootstrap.primordials;
  const { core } = window.__bootstrap;

  function sleep(milliseconds = 0) {
    // TODO(appcypher):
    return;
  }

  window.__bootstrap.timers = { sleep };
})(globalThis);
