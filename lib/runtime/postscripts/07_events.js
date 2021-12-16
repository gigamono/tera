// Copyright 2021 the Gigamono authors. All rights reserved. Apache 2.0 license.

"use strict";

((window) => {
  // Optional.
  if (window.__bootstrap.httpEvent == null) {
    return;
  }

  const {
    evGetRequestHeader,
    evSetRequestHeader,
    evGetRequestUriScheme,
    evGetRequestUriAuthority,
    evGetRequestUriPath,
    evGetRequestUriQuery,
    evGetRequestMethod,
    evGetRequestBodyReadStream,
    evReadRequestBodyChunk,
    evSetResponseHeader,
    evSetResponseStatus,
    evSetResponseVersion,
    evSetSendResponseBody,
    evSetSendResponseBodyWriteStream,
    evWriteResponseBodyChunk,
  } = window.__bootstrap.httpEvent;
  const { TypeError } = window.__bootstrap.primordials;
  const { Body, Response } = window.__bootstrap.http;

  class HttpEventRequest {
    constructor() {
      this.header = new HttpEventHeaders();
      this.uri = new HttpEventURI();
      this.body = new Body();

      this.body.setReadStream(() => {
        const rid = evGetRequestBodyReadStream();
        return async (buffer) => await evReadRequestBodyChunk(rid, buffer);
      });
    }

    get method() {
      evGetRequestMethod();
    }
  }

  class HttpEventHeaders {
    get(key) {
      return evGetRequestHeader(key);
    }

    set(key, value) {
      return evSetRequestHeader(key, value);
    }
  }

  class HttpEventURI {
    scheme() {
      return evGetRequestUriScheme();
    }

    authority() {
      return evGetRequestUriAuthority();
    }

    path() {
      return evGetRequestUriPath();
    }

    query() {
      return evGetRequestUriQuery();
    }
  }

  const http = {
    request: new HttpEventRequest(),
    respondWith: async function (response) {
      // Response object must be of type Response.
      if (!response instanceof Response) {
        throw new TypeError("expected response to be Response instance");
      }

      // Set headers if present.
      for (const [key, value] of response.headers) {
        evSetResponseHeader(key, value);
      }

      // Set status.
      evSetResponseStatus(response.status);

      // Set version.
      evSetResponseVersion(response.version);

      // If body contains a writeIterator, we use transfer-encoding chunked in Http/1.1 or streaming in H2.
      // This becomes Body(Streaming) in hyper.
      if (response.body.writeIterator) {
        // Set the write stream.
        response.body.setWriteStream(async () => {
          const rid = await evSetSendResponseBodyWriteStream();
          return async (buffer) => {
            await evWriteResponseBodyChunk(rid, buffer);
          };
        });

        // Drive the response body stream.
        await response.body.writeAll(response.body.writeIterator);
      } else {
        // If body contains a writeBuffer instead, we use fixed content-length body in H1 and H2.
        // This becomes Body(Full) in hyper.
        await evSetSendResponseBody(response.body.writeBuffer);
      }
    },
  };

  window.__bootstrap.events = { http };
})(globalThis);
