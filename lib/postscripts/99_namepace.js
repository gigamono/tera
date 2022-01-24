// Copyright 2021 the Gigamono authors. All rights reserved. GPL-3.0 License.

"use strict";

// Prevent setting __proto__. See https://github.com/nodejs/node/issues/31951
Object.defineProperty(Object.prototype, "__proto__", { set: void 0 });

((window) => {
  const { ObjectFreeze } = window.__bootstrap.primordials;
  const { core, files, events, errors, logger, encoding, http, __custom } =
    window.__bootstrap;

  // Register errors.
  core.registerErrorClass("NotSupported", errors.NotSupported);
  core.registerErrorClass("PermissionDenied", errors.PermissionDenied);
  core.registerErrorClass("Missing", errors.Missing);
  core.registerErrorClass("AssertionError", errors.AssertionError);
  core.registerErrorClass("UnimplementedError", errors.UnimplementedError);
  core.registerErrorClass("LimitExceededError", errors.LimitExceededError);

  // Delete other namespaces.
  delete window.__bootstrap;

  // SEC: This namespace is exposed to the user. Be explicit and careful about what you expose
  const Tera = {
    log: logger.log,
    encode: encoding.encode,
    decode: encoding.decode,
    Response: http.Response,
    File: files && files.File,
    events: events && events.events,
    ...__custom, // Custom extensions.
  };

  // Attach namespace to global.
  globalThis.Tera = Tera;

  // Freeze globalThis.Tera.
  ObjectFreeze(globalThis.Tera);
})(globalThis);
