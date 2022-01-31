// Copyright 2021 the Gigamono authors. All rights reserved. GPL-3.0 License.

"use strict";

((window) => {
  const { core } = window.__bootstrap;

  function envGet(key) {
    return core.opSync("opEnvGet", key);
  }

  window.__bootstrap.httpEvent = {
    envGet,
  };
})(globalThis);
