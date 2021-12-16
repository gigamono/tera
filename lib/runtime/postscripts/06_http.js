// Copyright 2021 the Gigamono authors. All rights reserved. Apache 2.0 license.

"use strict";

((window) => {
  const { ObjectEntries, ArrayBuffer, TypeError, Symbol } =
    window.__bootstrap.primordials;
  const { BufferStream } = window.__bootstrap.streams;
  const { encode } = window.__bootstrap.encoding;

  class Response {
    #headers = null;
    #status = null;
    #version = null;
    #body = null;

    constructor(body, options) {
      let opts = { headers: {}, status: 200, version: "1.1", ...options };

      this.#headers = new Headers(opts.headers);
      this.#status = new Status(opts.status);
      this.#version = new Version(opts.version);
      this.#body = new Body(body);
    }

    get body() {
      return this.#body;
    }

    get headers() {
      return this.#headers;
    }

    get version() {
      return this.#version.value;
    }

    get status() {
      return this.#status.value;
    }

    setHeader(k, v) {
      this.#headers.set(k, v);
    }

    set headers(value) {
      this.#headers = new Headers(value);
    }

    set version(value) {
      this.#version = new Version(value);
    }

    set status(value) {
      this.#status = new Status(value);
    }
  }

  class Headers {
    constructor(value) {
      this.value = value;
    }

    set(k, v) {
      this.value[k] = v;
    }

    get(k) {
      return this.value[k];
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

  class Body extends BufferStream {
    #writeBuffer = null; // Can either be a string, a typedarray or an iterator.
    #writeIterator = null;
    #readStreamCallback = null;
    #writeStreamCallback = null;

    constructor(value) {
      super();

      if (value == null) return;

      if (typeof value === "string") {
        this.#writeBuffer = encode(value);
      } else if (value instanceof ArrayBuffer) {
        this.#writeBuffer = value;
      } else if (Symbol.asyncIterator in value) {
        this.#writeIterator = value;
      } else {
        throw new TypeError(
          "expected body value to be a string, a typed array or an iterator"
        );
      }
    }

    get writeBuffer() {
      return this.#writeBuffer;
    }

    get writeIterator() {
      return this.#writeIterator;
    }

    setReadStream(readStreamCallback) {
      this.#readStreamCallback = readStreamCallback;
    }

    setWriteStream(writeStreamCallback) {
      this.#writeStreamCallback = writeStreamCallback;
    }

    async getReadStream() {
      return await this.#readStreamCallback();
    }

    async getWriteStream() {
      return await this.#writeStreamCallback();
    }
  }

  window.__bootstrap.http = { Body, Response };
})(globalThis);
