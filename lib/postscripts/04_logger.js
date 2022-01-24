// Copyright 2021 the Gigamono authors. All rights reserved. GPL-3.0 License.

"use strict";

((window) => {
  const { print } = window.__bootstrap.core;

  class Logger {
    static info(...messages) {
      print(`${messages.join(" ")}\n`);
    }

    static warn(...messages) {
      print(`${messages.join(" ")}\n`);
    }

    static error(...messages) {
      print(`${messages.join(" ")}\n`, true);
    }
  }

  const log = Logger;

  window.__bootstrap.logger = { log };
})(globalThis);
