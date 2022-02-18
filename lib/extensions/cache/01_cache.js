// Copyright 2021 the Gigamono authors. All rights reserved. GPL-3.0 License.

"use strict";

((window) => {
  const { core } = window.__bootstrap;

  function cacheGet(key) {
    return core.opSync("opCacheGet", key);
  }

  window.__bootstrap.httpEvent = {
    cacheGet,
  };
})(globalThis);
