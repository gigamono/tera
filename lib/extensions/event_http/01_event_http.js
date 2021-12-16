// Copyright 2021 the Gigamono authors. All rights reserved. Apache 2.0 license.

"use strict";

((window) => {
  const { core } = window.__bootstrap;

  function evGetRequestHeader(key) {
    return core.opSync("opEvGetRequestHeader", key);
  }

  function evSetRequestHeader(key, value) {
    return core.opSync("opEvSetRequestHeader", key, value);
  }

  function evGetRequestUriScheme() {
    return core.opSync("opEvGetRequestUriScheme");
  }

  function evGetRequestUriAuthority() {
    return core.opSync("opEvGetRequestUriAuthority");
  }

  function evGetRequestUriPath() {
    return core.opSync("opEvGetRequestUriPath");
  }

  function evGetRequestUriQuery() {
    return core.opSync("opEvGetRequestUriQuery");
  }

  function evGetRequestMethod() {
    return core.opSync("opEvGetRequestMethod");
  }

  function evGetRequestBodyReadStream() {
    return core.opSync("opEvGetRequestBodyReadStream");
  }

  async function evReadRequestBodyChunk(rid, buffer) {
    return core.opAsync("opEvReadRequestBodyChunk", rid, buffer);
  }

  function evGetRequestBodySizeHint(buffer) {
    return core.opAsync("opEvGetRequestBodySizeHint", buffer);
  }

  function evSetResponseHeader(key, value) {
    return core.opSync("opEvSetResponseHeader", key, value);
  }

  function evSetResponseHeader(key, value) {
    return core.opSync("opEvSetResponseHeader", key, value);
  }

  function evSetResponseStatus(status) {
    return core.opSync("opEvSetResponseStatus", status);
  }

  function evSetResponseVersion(version) {
    return core.opSync("opEvSetResponseVersion", version);
  }

  async function evSetSendResponseBody(buf) {
    return core.opAsync("opEvSetSendResponseBody", buf);
  }

  async function evSetSendResponseBodyWriteStream(buf) {
    return core.opAsync("opEvSetSendResponseBodyWriteStream", buf);
  }

  async function evWriteResponseBodyChunk(rid, buf) {
    return core.opAsync("opEvWriteResponseBodyChunk", rid, buf);
  }

  window.__bootstrap.httpEvent = {
    evGetRequestHeader,
    evSetRequestHeader,
    evGetRequestUriScheme,
    evGetRequestUriAuthority,
    evGetRequestUriPath,
    evGetRequestUriQuery,
    evGetRequestMethod,
    evGetRequestBodySizeHint,
    evGetRequestBodyReadStream,
    evReadRequestBodyChunk,
    evSetResponseHeader,
    evSetResponseStatus,
    evSetResponseVersion,
    evSetSendResponseBody,
    evSetSendResponseBodyWriteStream,
    evWriteResponseBodyChunk,
  };
})(globalThis);
