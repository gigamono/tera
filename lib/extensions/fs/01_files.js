"use strict";

((window) => {
  const core = window.__bootstrap.core;

  function readTextFile() {
    return core.opSync("op_read_text_file");
  }

  window.__bootstrap.files = { readTextFile };
})(globalThis);
