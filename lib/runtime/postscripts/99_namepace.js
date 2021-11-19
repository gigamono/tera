// Copyright 2021 the Gigamono authors. All rights reserved. Apache 2.0 license.

"use strict";

// Prevent setting __proto__. See https://github.com/nodejs/node/issues/31951
Object.defineProperty(Object.prototype, "__proto__", { set: void 0 });

((window) => {
  const { ObjectFreeze } = window.__bootstrap.primordials;
  const bootstrap = window.__bootstrap;

  // SEC: This namespace is exposed to the user. Be explicit and careful about what you expose.
  const sysNamespace = {
    // TODO
    // System FS
    // fs_open: bootstrap.fs.fs_open,
    // fs_close: bootstrap.fs.fs_close,
    // fs_read: bootstrap.fs.fs_read,
    // fs_write: bootstrap.fs.fs_write,
    // IO
    // read: bootstrap.io.read
    // readSync: bootstrap.io.read
    // readAll: bootstrap.io.readAll,
    // readAllSync: bootstrap.io.readAll,
    // writeAll: bootstrap.io.writeAll,
    // writeAllSync: bootstrap.io.writeAll,
    // iter: bootstrap.io.iter,
    // iterSync: bootstrap.io.iterSync,
    // copy: bootstrap.io.copy
    // Files
    // File: bootstrap.files.File, // .read, .readAll, .write, .writeAll
    // Console
    // console: bootstrap.console.conosle, // .log, .error, .trace, .warn
    // HTTPEvent
    // httpEvent: bootstrap.httpEvent.httpEvent, // .request, .respondWith
  };

  // Delete other namespaces.
  // Removing the Deno namespace because there is a possibility of divergence in the future.
  delete window.__bootstrap;
  delete window.Deno;

  // Create "sys" namespace.
  globalThis.sys = sysNamespace;

  // Freeze namespace object.
  ObjectFreeze(globalThis.sys);
})(globalThis);
