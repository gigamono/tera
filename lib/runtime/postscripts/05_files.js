// Copyright 2021 the Gigamono authors. All rights reserved. Apache 2.0 license.
// Based on https://github.com/denoland/deno/tree/main/runtime. Copyright the Deno authors. MIT license.

"use strict";

((window) => {
  // Optional.
  if (window.__bootstrap.fs == null) {
    return;
  }

  const { fs_open, fs_read, fs_write } = window.__bootstrap.fs;
  const { Stream } = window.__bootstrap.streams;

  class File extends Stream {
    contructor(rid) {
      this.rid = rid;
    }

    static async open(path, options = {}) {
      const rid = await fs_open(path, options);
      return new File(rid);
    }

    get_read_stream() {
      return async (buffer) => await fs_read(this.rid, buffer);
    }

    get_write_stream() {
      return async (buffer) => await fs_write(this.rid, buffer);
    }
  }

  window.__bootstrap.files = { File };
})(globalThis);
