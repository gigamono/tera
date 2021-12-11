// Copyright 2021 the Gigamono authors. All rights reserved. Apache 2.0 license.

"use strict";

((window) => {
  const core = window.__bootstrap.core;

  function open(path, options) {
    return core.opAsync("op_open", path, options);
  }

  function fs_read(resourceId, buf) {
    return core.opAsync("op_fs_read", resourceId, buf);
  }

  function fs_write(resourceId, buf) {
    return core.opAsync("op_fs_write", resourceId, buf);
  }

  function fs_seek(resourceId, buf) {
    return core.opAsync("op_fs_seek", resourceId, buf);
  }

  window.__bootstrap.fs = { open, fs_read, fs_write, fs_seek };
})(globalThis);
