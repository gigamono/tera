// Copyright 2021 the Gigamono authors. All rights reserved. Apache 2.0 license.
// Based on https://github.com/denoland/deno/tree/main/runtime. Copyright the Deno authors. MIT license.

"use strict";

((window) => {
  // Optional.
  if (window.__bootstrap.fs == null) {
    return;
  }

  const { fsOpen, fsRead, fsWrite } = window.__bootstrap.fs;
  const { BufferStream } = window.__bootstrap.streams;

  class File extends BufferStream {
    #rid = Number.MAX_SAFE_INTEGER;
    #path = null;

    constructor(rid, path) {
      super();
      this.#rid = rid;
      this.#path = path;
    }

    static async open(path, options = {}) {
      const rid = await fsOpen(path, options);
      return new File(rid, path);
    }

    get rid() {
      return this.#rid;
    }

    get path() {
      return this.#path;
    }

    async getReadStream() {
      return async (buffer) => await fsRead(this.#rid, buffer);
    }

    async getWriteStream() {
      return async (buffer) => await fsWrite(this.#rid, buffer);
    }
  }

  window.__bootstrap.files = { File };
})(globalThis);
