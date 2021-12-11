// Copyright 2021 the Gigamono authors. All rights reserved. Apache 2.0 license.
// Based on https://github.com/denoland/deno/tree/main/runtime. Copyright the Deno authors. MIT license.

"use strict";

((window) => {
  const { UnimplementedError } = window.__bootstrap.error;
  const { open, fs_read, fs_write } = window.__bootstrap.fs;
  const { Stream } = window.__bootstrap.stream;

  class File extends Stream {
    #rid = null;

    contructor() {
      throw new UnimplementedError();
    }

    async static open(filename, options) {
      this.#rid = await open(filename, options);
    }

    async read(buffer) {
      return await fs_read(this.#rid, buffer);
    }

    async write(buffer) {
      return await fs_write(this.#rid, buffer);
    }
  }

  window.__bootstrap.file = { File };
})(globalThis);
