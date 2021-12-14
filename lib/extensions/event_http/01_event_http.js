// Copyright 2021 the Gigamono authors. All rights reserved. Apache 2.0 license.

"use strict";

((window) => {
  const { core } = window.__bootstrap;

  function ev_get_request_header(key) {
    return core.opSync("op_ev_get_request_header", key);
  }

  function ev_set_request_header(key, value) {
    return core.opSync("op_ev_set_request_header", key, value);
  }

  function ev_get_request_uri_scheme() {
    return core.opSync("op_ev_get_request_uri_scheme");
  }

  function ev_get_request_uri_authority() {
    return core.opSync("op_ev_get_request_uri_authority");
  }

  function ev_get_request_uri_path() {
    return core.opSync("op_ev_get_request_uri_path");
  }

  function ev_get_request_uri_query() {
    return core.opSync("op_ev_get_request_uri_query");
  }

  function ev_get_request_method() {
    return core.opSync("op_ev_get_request_method");
  }

  function ev_get_request_body_stream() {
    return core.opSync("op_ev_get_request_body_stream");
  }

  async function ev_read_request_body_chunk(rid, buffer) {
    return core.opAsync("op_ev_read_request_body_chunk", rid, buffer);
  }

  function ev_get_request_body_size_hint(buffer) {
    return core.opAsync("op_ev_get_request_body_size_hint", buffer);
  }

  function ev_set_response_header(key, value) {
    return core.opSync("op_ev_set_response_header", key, value);
  }

  function ev_set_response_header(key, value) {
    return core.opSync("op_ev_set_response_header", key, value);
  }

  function ev_set_response_status(status) {
    return core.opSync("op_ev_set_response_status", status);
  }

  function ev_set_response_version(version) {
    return core.opSync("op_ev_set_response_version", version);
  }

  function ev_write_response_body(buf) {
    return core.opSync("op_ev_write_response_body", buf);
  }

  async function ev_send_response() {
    return core.opAsync("op_ev_send_response");
  }

  window.__bootstrap.httpEvent = {
    ev_get_request_header,
    ev_set_request_header,
    ev_get_request_uri_scheme,
    ev_get_request_uri_authority,
    ev_get_request_uri_path,
    ev_get_request_uri_query,
    ev_get_request_method,
    ev_get_request_body_size_hint,
    ev_get_request_body_stream,
    ev_read_request_body_chunk,
    ev_set_response_header,
    ev_set_response_status,
    ev_set_response_version,
    ev_write_response_body,
    ev_send_response,
  };
})(globalThis);
