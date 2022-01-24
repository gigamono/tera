// Copyright 2021 the Gigamono authors. All rights reserved. GPL-3.0 License.

"use strict";

((window) => {
  const { core } = window.__bootstrap;

  function fsOpen(path, options) {
    return core.opAsync("opFsOpen", path, options);
  }

  function fsRead(rid, buf) {
    return core.opAsync("opFsRead", rid, buf);
  }

  function fsWrite(rid, buf) {
    return core.opAsync("opFsWrite", rid, buf);
  }

  function fsSeek(rid, buf) {
    return core.opAsync("opFsSeek", rid, buf);
  }

  window.__bootstrap.fs = { fsOpen, fsRead, fsWrite, fsSeek };
})(globalThis);
