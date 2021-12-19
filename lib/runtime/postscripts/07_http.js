// Copyright 2021 the Gigamono authors. All rights reserved. Apache 2.0 license.

"use strict";

((window) => {
  const { ObjectEntries, ArrayBuffer, TypeError, Symbol } =
    window.__bootstrap.primordials;
  const { BufferStream } = window.__bootstrap.streams;
  const { encode } = window.__bootstrap.encoding;
  const { File } = window.__bootstrap.files;

  class Response {
    #headers = null;
    #status = null;
    #version = null;
    #body = null;

    constructor(body, options = {}) {
      this.#status = new Status(options.status || 200);
      this.#version = new Version(options.version || "1.1");
      this.#body = new Body(body);

      // Guess content type.
      const contentType = this.#body.guessContentType();
      this.#headers = new Headers({
        ...(contentType && { "Content-Type": contentType }),
        ...(options.headers || {}),
      });
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
    #object = {};

    constructor(object) {
      if (object == null) {
        throw new TypeError("expected parameter to be an object");
      }

      this.#object = object;
    }

    get value() {
      return this.#object;
    }

    get(k) {
      return this.#object[k];
    }

    set(k, v) {
      this.#object[k] = v;
    }

    *[Symbol.iterator]() {
      for (const [k, v] of ObjectEntries(this.#object)) {
        yield [k, v];
      }
    }

    toString() {
      return JSON.stringify(this.value);
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
    #writeType = "string"; // 'string', 'typedArray', 'asyncIterator', 'file'.
    #writeObject = null;
    #path = null;
    #readStreamCallback = null;
    #writeStreamCallback = null;

    constructor(object) {
      super();

      if (object == null) return;

      if (typeof object === "string") {
        this.#writeObject = encode(object);
      } else if (object instanceof ArrayBuffer) {
        this.#writeObject = object;
        this.#writeType = "typedArray";
      } else if (object instanceof File) {
        // This check has to be on top because File has Symbol.asyncIterator.
        // Create an asyncIterator from file.
        async function* iterateFile(file) {
          for await (const buf of file) {
            yield buf;
          }
        }

        this.#writeObject = iterateFile(object);
        this.#writeType = "file";
        this.#path = object.path;
      } else if (Symbol.asyncIterator in object) {
        this.#writeObject = object;
        this.#writeType = "asyncIterator";
      } else {
        throw new TypeError(
          "expected body value to be a string, a typed array or an iterator"
        );
      }
    }

    get writeType() {
      return this.#writeType;
    }

    get writeObject() {
      return this.#writeObject;
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

    guessContentType() {
      let contentType = null;

      switch (this.#writeType) {
        case "file": {
          let ext = "";

          const matches = this.#path.match(/(?<=\.).+$/);
          if (matches) {
            ext = matches[0];
          }

          contentType = this.#guessFileContentType(ext);
          break;
        }
        case "typedArray":
        case "asyncIterator": {
          contentType = "application/octet-stream";
        }
      }

      return contentType;
    }

    #guessFileContentType(ext) {
      // https://developer.mozilla.org/en-US/docs/Web/HTTP/Basics_of_HTTP/MIME_types/Common_types
      switch (ext) {
        // IMAGE
        case "svg":
          return "image/svg+xml";
        case "png":
          return "image/png";
        case "webp":
          return "image/webp";
        case "jpg":
        case "jpeg":
          return "image/jpeg";
        case "gif":
          return "image/gif";
        case "bmp":
          return "image/bmp";
        // TEXT
        case "txt":
          return "text/plain";
        case "csv":
          return "text/csv";
        case "js":
          return "text/javascript";
        case "css":
          return "text/css";
        case "htm":
        case "html":
          return "text/html";
        case "ics":
          return "text/calendar";
        // APPLICATION
        case "pdf":
          return "application/pdf";
        case "abw":
          return "application/x-abiword";
        case "arc":
          return "application/x-freearc";
        case "xml":
          return "application/xml";
        case "xhtml":
          return "application/xhtml+xml";
        case "zip":
          return "application/zip";
        case "7z":
          return "application/x-7z-compressed";
        case "gz":
          return "application/gzip";
        case "rar":
          return "application/vnd.rar";
        case "tar":
          return "application/x-tar";
        case "bz":
          return "application/x-bzip";
        case "bz2":
          return "application/x-bzip2";
        case "epub":
          return "application/epub+zip";
        case "bin":
          return "application/octet-stream";
        case "ppt":
          return "application/vnd.ms-powerpoint";
        case "pptx":
          return "application/vnd.openxmlformats-officedocument.presentationml.presentation";
        case "xls":
          return "application/vnd.ms-excel";
        case "xlsx":
          return "application/vnd.openxmlformats-officedocument.spreadsheetml.sheet";
        case "doc":
          return "application/msword";
        case "docx":
          return "application/vnd.openxmlformats-officedocument.wordprocessingml.document";
        case "rtf":
          return "application/rtf";
        case "eot":
          return "application/vnd.ms-fontobject";
        case "jar":
          return "application/java-archive";
        case "json":
          return "application/json";
        case "jsonld":
          return "application/ld+json";
        case "ogx":
          return "application/ogg";
        case "php":
          return "application/x-httpd-php";
        case "swf":
          return "application/x-shockwave-flash";
        case "xul":
          return "application/vnd.mozilla.xul+xml";
        case "sh":
          return "application/x-sh";
        // FONT
        case "ttf":
          return "font/ttf";
        case "otf":
          return "font/otf";
        case "woff":
          return "font/woff";
        case "woff2":
          return "font/woff2";
        // AUDIO
        case "mp3":
          return "audio/mpeg";
        case "oga":
          return "audio/ogg";
        case "aac":
          return "audio/aac";
        case "wav":
          return "audio/wav";
        case "opus":
          return "audio/opus";
        case "weba":
        case "webm":
          return "audio/webm";
        case "mid":
        case "midi":
          return "audio/midi";
        // VIDEO
        case "mpeg":
          return "video/mpeg";
        case "mp4":
          return "video/mp4";
        case "ts":
          return "video/mp2t";
        case "avi":
          return "video/x-msvideo";
        case "ogv":
          return "video/ogg";
        default:
          return null;
      }
    }
  }

  window.__bootstrap.http = { Body, Response };
})(globalThis);
