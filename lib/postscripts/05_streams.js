// Copyright 2021 the Gigamono authors. All rights reserved. Apache 2.0 license.

"use strict";

((window) => {
  const {
    Uint8Array,
    ArrayPrototypePush,
    TypedArrayPrototypeSet,
    TypedArrayPrototypeSubarray,
    TypeError,
    TypedArray,
    Symbol,
  } = window.__bootstrap.primordials;
  const { UnimplementedError, LimitExceededError } = window.__bootstrap.errors;
  const { SIZE_PER_ITER } = window.__bootstrap.common;

  class BufferStream {
    async getWriteStream() {
      return null;
    }

    async getReadStream() {
      return null;
    }

    close() {
      throw new UnimplementedError();
    }

    // Writes a buffer to destination. Writes large buffers in SIZE_PER_ITER chunks.
    async writeAll(buffer, bufferSize = SIZE_PER_ITER) {
      if (bufferSize > SIZE_PER_ITER) {
        throw new LimitExceededError(
          `buffer size cannot be greater than ${SIZE_PER_ITER} bytes`
        );
      }

      const write = await this.getWriteStream();
      if (write == null) {
        return;
      }

      if (buffer instanceof TypedArray) {
        await this.#writeAllBuffer(write, buffer, bufferSize);
      } else if (Symbol.asyncIterator in buffer) {
        await this.#writeAllIterator(write, buffer, bufferSize);
      } else {
        throw new TypeError(
          "expected buffer to be a typed array or an iterator"
        );
      }
    }

    async #writeAllBuffer(write, buffer, bufferSize) {
      let totalWritten = 0;

      while (totalWritten < buffer.length) {
        totalWritten += await write(
          TypedArrayPrototypeSubarray(buffer, totalWritten, bufferSize)
        );
      }
    }

    async #writeAllIterator(write, generator, bufferSize) {
      for await (const buf of generator) {
        // Generated buffer must be within 0 to bufferSize.
        if (buf.length > bufferSize) {
          throw new LimitExceededError(
            `generated buffer cannot be greater than ${bufferSize} bytes`
          );
        }

        await write(buf);
      }

      // Last empty buffer write for streams that require it to close.
      await write(new Uint8Array(0));
    }

    // Reads SIZE_PER_ITER chunks from source and concats them into a single large buffer.
    async readAll(bufferSize = SIZE_PER_ITER) {
      if (bufferSize > SIZE_PER_ITER) {
        throw new LimitExceededError(
          `buffer size cannot be greater than ${SIZE_PER_ITER} bytes`
        );
      }

      const read = await this.getReadStream();
      if (read == null) {
        return Uint8Array(0);
      }

      const buffers = [];
      let totalSize = 0;

      while (true) {
        const buffer = new Uint8Array(bufferSize);
        const totalRead = await read(buffer);
        totalSize += totalRead;

        if (totalRead === 0) {
          break;
        }

        ArrayPrototypePush(
          buffers,
          TypedArrayPrototypeSubarray(buffer, 0, totalRead)
        );
      }

      return this.#concatBuffers(buffers, totalSize);
    }

    async *[Symbol.asyncIterator]() {
      const read = await this.getReadStream();
      if (read == null) {
        return;
      }

      while (true) {
        const buffer = new Uint8Array(SIZE_PER_ITER);
        const totalRead = await read(buffer);

        if (totalRead === 0) {
          return;
        }

        yield TypedArrayPrototypeSubarray(buffer, 0, totalRead);
      }
    }

    #concatBuffers(buffers, totalSize) {
      const bigBuffer = new Uint8Array(totalSize);
      let offset = 0;

      for (const buffer of buffers) {
        TypedArrayPrototypeSet(bigBuffer, buffer, offset);
        offset += buffer.length;
      }

      return bigBuffer;
    }
  }

  window.__bootstrap.streams = { BufferStream };
})(globalThis);
