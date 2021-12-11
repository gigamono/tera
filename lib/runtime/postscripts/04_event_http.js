// Copyright 2021 the Gigamono authors. All rights reserved. Apache 2.0 license.

"use strict";

((window) => {
  const {
    get_request_header,
    set_request_header,
    get_request_uri_scheme,
    get_request_uri_authority,
    get_request_uri_path,
    get_request_uri_query,
    get_request_method,
    read_request_body,
    set_response_header,
    set_response_status,
    set_response_version,
    write_response_body,
    send_response,
  } = window.__bootstrap.httpEvent;
  const { Stream } = window.__bootstrap.stream;

  class EventRequest {
    constructor() {
      this.header = new Headers();
      this.uri = new URI();
      this.body = new Body();
    }

    get method() {
      get_request_method();
    }
  }

  class Headers {
    get(key) {
      return get_request_header(key);
    }

    set(key, value) {
      return set_request_header(key, value);
    }
  }

  class URI {
    scheme() {
      return get_request_uri_scheme();
    }

    authority() {
      return get_request_uri_authority();
    }

    path() {
      return get_request_uri_path();
    }

    query() {
      return get_request_uri_query();
    }
  }

  class Body extends Stream {
    async read(buffer) {
      await read_request_body(buffer);
    }
  }

  const httpEvent = {
    request: new EventRequest(),
    sendResponse: async function (body, { headers, status, version }) {
      // Set headers.
      for (const [key, value] of ObjectEntries(headers)) {
        set_response_header(key, value);
      }

      // Set status.
      set_response_status(status);

      // Set version.
      set_response_version(version);

      // TODO(appcypher): Fix body streaming.
      write_response_body(body);

      // Send response.
      await send_response(response);
    },
  };

  window.__bootstrap.httpEvent = { httpEvent };
})(globalThis);
