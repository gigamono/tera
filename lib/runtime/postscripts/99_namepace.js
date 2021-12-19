// Copyright 2021 the Gigamono authors. All rights reserved. Apache 2.0 license.

"use strict";

// Prevent setting __proto__. See https://github.com/nodejs/node/issues/31951
Object.defineProperty(Object.prototype, "__proto__", { set: void 0 });

((window) => {
  const { ObjectFreeze } = window.__bootstrap.primordials;
  const { core, files, events, errors, logger, encoding, http } =
    window.__bootstrap;

  // Register errors.
  core.registerErrorClass("NotSupported", errors.NotSupported);
  core.registerErrorClass("PermissionDenied", errors.PermissionDenied);
  core.registerErrorClass("Missing", errors.Missing);
  core.registerErrorClass("UnimplementedError", errors.UnimplementedError);

  // Delete other namespaces.
  // Removing the Deno namespace to prevent confusion. The bindings are not always compatible.
  delete window.__bootstrap;
  delete window.Deno;

  // TODO(appcypher): Support user-supplied namespace name.
  // SEC: This namespace is exposed to the user. Be explicit and careful about what you expose
  // Destructuring does not work as intended.
  {
    globalThis.log = logger.log;
    globalThis.encode = encoding.encode;
    globalThis.decode = encoding.decode;
    globalThis.Response = http.Response;

    if (files) {
      globalThis.File = files.File;
    }

    if (events) {
      globalThis.events = events.events;
    }
  }

  // Freeze globalThis.
  ObjectFreeze(globalThis);
})(globalThis);
