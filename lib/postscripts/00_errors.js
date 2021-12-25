// Copyright 2021 the Gigamono authors. All rights reserved. Apache 2.0 license.
// Based on https://github.com/denoland/deno/tree/main/runtime. Copyright the Deno authors. MIT license.

"use strict";

((window) => {
  const { Error } = window.__bootstrap.primordials;

  class PermissionDenied extends Error {
    constructor(message) {
      super(message);
      this.name = "PermissionDenied";
    }
  }

  class NotSupported extends Error {
    constructor(message) {
      super(message);
      this.name = "NotSupported";
    }
  }

  class Missing extends Error {
    constructor(message) {
      super(message);
      this.name = "Missing";
    }
  }

  class AssertionError extends Error {
    constructor(msg) {
      super(msg);
      this.name = "AssertionError";
    }
  }

  class UnimplementedError extends Error {
    constructor(msg) {
      super(msg);
      this.name = "UnimplementedError";
    }
  }

  class LimitExceededError extends Error {
    constructor(msg) {
      super(msg);
      this.name = "LimitExceededError";
    }
  }

  window.__bootstrap.errors = {
    PermissionDenied,
    NotSupported,
    Missing,
    AssertionError,
    UnimplementedError,
    LimitExceededError,
  };
})(globalThis);
