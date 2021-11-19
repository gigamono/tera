// Copyright 2021 the Gigamono authors. All rights reserved. Apache 2.0 license.

"use strict";

((window) => {
  const core = window.__bootstrap.core;

  function readTextFile(path) {
    return core.opAsync("op_read_text_file", path);
  }

  function open(path, options) {
    return core.opAsync("op_open", path, options);
  }

  function writeAll(path, content) {
    return core.opAsync("op_write_all", path, content);
  }

  window.__bootstrap.files = { readTextFile, open, writeAll };
})(globalThis);
