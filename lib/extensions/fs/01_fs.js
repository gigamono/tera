// Copyright 2021 the Gigamono authors. All rights reserved. Apache 2.0 license.

"use strict";

((window) => {
  const { core } = window.__bootstrap;

  function fs_open(path, options) {
    return core.opAsync("op_fs_open", path, options);
  }

  function fs_read(rid, buf) {
    return core.opAsync("op_fs_read", rid, buf);
  }

  function fs_write(rid, buf) {
    return core.opAsync("op_fs_write", rid, buf);
  }

  function fs_seek(rid, buf) {
    return core.opAsync("op_fs_seek", rid, buf);
  }

  window.__bootstrap.fs = { fs_open, fs_read, fs_write, fs_seek };
})(globalThis);
