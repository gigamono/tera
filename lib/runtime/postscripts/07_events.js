// Copyright 2021 the Gigamono authors. All rights reserved. Apache 2.0 license.

"use strict";

((window) => {
  // Optional.
  if (window.__bootstrap.httpEvent == null) {
    return;
  }

  const {
    ev_get_request_header,
    ev_set_request_header,
    ev_get_request_uri_scheme,
    ev_get_request_uri_authority,
    ev_get_request_uri_path,
    ev_get_request_uri_query,
    ev_get_request_method,
    ev_get_request_body_stream,
    ev_read_request_body_chunk,
    ev_set_response_header,
    ev_set_response_status,
    ev_set_response_version,
    ev_write_response_body,
    ev_send_response,
  } = window.__bootstrap.httpEvent;
  const { Stream } = window.__bootstrap.streams;
  const { log } = window.__bootstrap.logger;

  class HttpEventRequest {
    constructor() {
      this.header = new HttpEventHeaders();
      this.uri = new HttpEventURI();
      this.body = new HttpEventBody();
    }

    get method() {
      ev_get_request_method();
    }
  }

  class HttpEventHeaders {
    get(key) {
      return ev_get_request_header(key);
    }

    set(key, value) {
      return ev_set_request_header(key, value);
    }
  }

  class HttpEventURI {
    scheme() {
      return ev_get_request_uri_scheme();
    }

    authority() {
      return ev_get_request_uri_authority();
    }

    path() {
      return ev_get_request_uri_path();
    }

    query() {
      return ev_get_request_uri_query();
    }
  }

  class HttpEventBody extends Stream {
    get_read_stream() {
      const rid = ev_get_request_body_stream();

      return async (buffer) => await ev_read_request_body_chunk(rid, buffer);
    }
  }

  const http = {
    request: new HttpEventRequest(),
    respondWith: async function (response) {
      // Set headers if present.
      for (const [key, value] of response.headers()) {
        ev_set_response_header(key, value);
      }

      log.info(">>> response =", response);

      // Set status.
      ev_set_response_status(response.status());

      // Set version.
      ev_set_response_version(response.version());

      // Write response body.
      ev_write_response_body(response.body());

      // Send response.
      await ev_send_response();
    },
  };

  window.__bootstrap.events = { http };
})(globalThis);
