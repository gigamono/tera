"use strict";

((window) => {
  const core = window.__bootstrap.core;

  function readTextFile(path) {
    return core.opAsync("op_read_text_file", path);
  }

  window.__bootstrap.files = { readTextFile };
})(globalThis);
