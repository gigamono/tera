// Copyright 2021 the Gigamono authors. All rights reserved. Apache 2.0 license.

"use strict";

((window) => {
  const { ObjectEntries } = window.__bootstrap.primordials;

  class Response {
    constructor(body, options) {
      let opts = { headers: {}, status: 200, version: "1.1", ...options };

      if (typeof body == "string") {
        body = encode(body);
      }

      this._headers = new Headers(opts.headers);
      this._status = new Status(opts.status);
      this._version = new Version(opts.version);
      this._body = new Body(body);
    }

    body() {
      return this._body.value;
    }

    headers() {
      return this._headers;
    }

    version() {
      return this._version.value;
    }

    status() {
      return this._status.value;
    }

    set_header(k, v) {
      this._headers.set(k, v);
    }

    set_headers(value) {
      this._headers = new Headers(value);
    }

    set_version(value) {
      this._version = new Version(value);
    }

    set_status(value) {
      this._status = new Status(value);
    }
  }

  class Headers {
    constructor(value) {
      this.value = value;
    }

    set(k, v) {
      this.value[k] = v;
    }

    *[Symbol.iterator]() {
      for (const [k, v] of ObjectEntries(this.value)) {
        yield [k, v];
      }
    }
  }

  class Status {
    constructor(value) {
      this.value = value;
    }
  }

  class Version {
    constructor(value) {
      this.value = value;
    }
  }

  class Body {
    constructor(value) {
      this.value = value;
    }
  }

  window.__bootstrap.http = { Response };
})(globalThis);
