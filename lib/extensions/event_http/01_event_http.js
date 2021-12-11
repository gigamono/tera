// Copyright 2021 the Gigamono authors. All rights reserved. Apache 2.0 license.

"use strict";

((window) => {
  const core = window.__bootstrap.core;

  function get_request_header(key) {
    return core.opSync("op_get_request_header", key);
  }

  function set_request_header(key, value) {
    return core.opSync("op_set_request_header", key, value);
  }

  function get_request_uri_scheme() {
    return core.opSync("op_get_request_uri_scheme");
  }

  function get_request_uri_authority() {
    return core.opSync("op_get_request_uri_authority");
  }

  function get_request_uri_path() {
    return core.opSync("op_get_request_uri_path");
  }

  function get_request_uri_query() {
    return core.opSync("op_get_request_uri_query");
  }

  function get_request_method() {
    return core.opSync("op_get_request_method");
  }

  function read_request_body() {
    return core.opAsync("op_read_request_body");
  }

  function set_response_header(key, value) {
    return core.opSync("op_set_response_header", key, value);
  }

  function set_response_header(key, value) {
    return core.opSync("op_set_response_header", key, value);
  }

  function set_response_status(status) {
    return core.opSync("op_set_response_status", status);
  }

  function set_response_version(version) {
    return core.opSync("op_set_response_version", version);
  }

  function write_response_body(buf) {
    return core.opSync("op_write_response_body", buf);
  }

  function send_response() {
    return core.opync("op_send_response");
  }

  window.__bootstrap.httpEvent = {
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
  };
})(globalThis);
