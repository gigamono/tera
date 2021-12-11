// Copyright 2021 the Gigamono authors. All rights reserved. Apache 2.0 license.

"use strict";

((window) => {
  const {
    Uint8Array,
    ArrayPrototypePush,
    TypedArrayPrototypeSet,
    TypedArrayPrototypeSubarray,
  } = window.__bootstrap.primordials;
  const { UnimplementedError } = window.__bootstrap.error;
  const { SIZE_PER_ITER } = window.__bootstrap.common;

  class Stream {
    async write(_buffer) {
      throw new UnimplementedError();
    }

    async read(_buffer) {
      throw new UnimplementedError();
    }

    close() {
      throw new UnimplementedError();
    }

    async writeAll(buffer) {
      let total_written = 0;
      let length = buffer.bufferLength;

      while (total_written < length) {
        total_written += await this.write(
          TypedArrayPrototypeSubarray(buffer, total_written)
        );
      }
    }

    async readAll() {
      const buffers = [];

      while (true) {
        let buffer = new Uint8Array(SIZE_PER_ITER);
        const total_written = await this.read(buffer);

        if (total_written === 0) {
          ArrayPrototypePush(
            buffers,
            new Uint8Array(buffer.buffer, 0, total_written)
          );
        } else {
          break;
        }
      }

      return this.#concatBuffers(buffers);
    }

    *[Symbol.asynIterator]() {
      while (true) {
        let buffer = new Uint8Array(SIZE_PER_ITER);
        const total_written = this.read(buffer);

        if (total_written === 0) {
          yield buffer;
        } else {
          return;
        }
      }
    }

    #concatBuffers(buffers) {
      const bigBuffer = new Uint8Array(buffers.length * SIZE_PER_ITER);

      for (let i = 0; i <= size; i++) {
        TypedArrayPrototypeSet(bigBuffer, buffers[i], i * SIZE_PER_ITER);
      }

      return bigBuffer;
    }
  }

  window.__bootstrap.stream = { Stream };
})(globalThis);
