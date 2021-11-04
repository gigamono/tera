"use strict";

// Prevent setting __proto__. See https://github.com/nodejs/node/issues/31951
Object.defineProperty(Object.prototype, "__proto__", { set: void 0 });

((window) => {
  const { ObjectFreeze } = window.__bootstrap.primordials;
  const bootstrap = window.__bootstrap;

  // SEC: This namespace is exposed to the user. Be explicit and careful what you expose.
  const sysNamespace = {
    core: {
      print: bootstrap.core.print,
    },
    readTextFile: bootstrap.files.readTextFile,
    open: bootstrap.files.open,
    writeAll: bootstrap.files.writeAll,
  };

  // Create "sys" namespace and freeze it.
  globalThis.sys = sysNamespace;
  ObjectFreeze(globalThis.sys);

  // Delete other namespaces.
  delete window.__bootstrap;
  delete window.Deno;
})(globalThis);
