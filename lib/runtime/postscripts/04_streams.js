// Copyright 2021 the Gigamono authors. All rights reserved. Apache 2.0 license.

"use strict";

((window) => {
  const {
    Uint8Array,
    ArrayPrototypePush,
    TypedArrayPrototypeSet,
    TypedArrayPrototypeSubarray,
  } = window.__bootstrap.primordials;
  const { UnimplementedError } = window.__bootstrap.errors;
  const { SIZE_PER_ITER } = window.__bootstrap.common;

  class Stream {
    get_write_stream() {
      return null;
    }

    get_read_stream() {
      return null;
    }

    close() {
      throw new UnimplementedError();
    }

    async writeAll(buffer) {
      const write = this.get_write_stream();

      if (write == null) {
        return Uint8Array(0);
      }

      let total_written = 0;
      const length = buffer.bufferLength;

      while (total_written < length) {
        total_written += await write(
          TypedArrayPrototypeSubarray(buffer, total_written)
        );
      }
    }

    async readAll(buffer_size = SIZE_PER_ITER) {
      const read = this.get_read_stream();
      if (read == null) {
        return Uint8Array(0);
      }

      const buffers = [];
      let total_size = 0;

      while (true) {
        const buffer = new Uint8Array(buffer_size);
        const total_read = await read(buffer);
        total_size += total_read;

        if (total_read !== 0) {
          ArrayPrototypePush(buffers, buffer.subarray(0, total_read));
        } else {
          break;
        }
      }

      return this.#concatBuffers(buffers, total_size);
    }

    async *[Symbol.asyncIterator]() {
      const read = this.get_read_stream();

      if (read == null) {
        return;
      }

      while (true) {
        const buffer = new Uint8Array(SIZE_PER_ITER);
        const total_read = await read(buffer);

        if (total_read !== 0) {
          yield buffer.subarray(0, total_read);
        } else {
          return;
        }
      }
    }

    #concatBuffers(buffers, total_size) {
      const bigBuffer = new Uint8Array(total_size);
      let offset = 0;

      for (const buffer of buffers) {
        TypedArrayPrototypeSet(bigBuffer, buffer, offset);
        offset += buffer.length;
      }

      return bigBuffer;
    }
  }

  window.__bootstrap.streams = { Stream };
})(globalThis);
