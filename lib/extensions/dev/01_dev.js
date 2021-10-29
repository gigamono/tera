"use strict";

((window) => {
  const core = window.__bootstrap.core;

  function devSync() {
    return core.opSync("op_dev_sync");
  }

  async function devAsync() {
    return await core.opSync("op_dev_async");
  }

  window.__bootstrap.dev = { devSync, devAsync };
})(globalThis);
