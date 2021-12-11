// Copyright 2021 the Gigamono authors. All rights reserved. Apache 2.0 license.

"use strict";

// Prevent setting __proto__. See https://github.com/nodejs/node/issues/31951
Object.defineProperty(Object.prototype, "__proto__", { set: void 0 });

((window) => {
  const { ObjectFreeze } = window.__bootstrap.primordials;
  const error = window.__bootstrap.error;

  // Register errors.
  core.registerErrorClass("NotSupported", error.NotSupported);
  core.registerErrorClass("PermissionDenied", error.PermissionDenied);
  core.registerErrorClass("Missing", error.Missing);

  // SEC: This namespace is exposed to the user. Be explicit and careful about what you expose.
  const bootstrap = window.__bootstrap;
  const namespace = {
    // === File ===
    File: bootstrap.file.File,

    // === HTTPEvent ===
    httpEvent: bootstrap.httpEvent.httpEvent,

    // === Console ===
    // console: bootstrap.console.console,
  };

  // Delete other namespaces.
  // Removing the Deno namespace to prevent confusion. The bindings are not always compatible.
  delete window.__bootstrap;
  delete window.Deno;

  // TODO(appcypher): Support user-supplied namespace name.
  globalThis = namespace;

  // Freeze namespace object.
  ObjectFreeze(globalThis.sys);
})(globalThis);
