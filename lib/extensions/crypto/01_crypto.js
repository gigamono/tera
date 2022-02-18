// Copyright 2021 the Gigamono authors. All rights reserved. GPL-3.0 License.

"use strict";

((window) => {
  const { core } = window.__bootstrap;

  function cryptoCreateHmac(key) {
    return core.opSync("opCryptoCreateHmac", key);
  }

  window.__bootstrap.httpEvent = {
    cryptoCreateHmac,
  };
})(globalThis);
