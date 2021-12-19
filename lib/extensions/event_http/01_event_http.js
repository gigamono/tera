// Copyright 2021 the Gigamono authors. All rights reserved. Apache 2.0 license.

"use strict";

((window) => {
  const { core } = window.__bootstrap;

  function httpGetRequestHeaders(key) {
    return core.opSync("opEvGetRequestHeaders", key);
  }

  function httpGetRequestHeader(key) {
    return core.opSync("opEvGetRequestHeader", key);
  }

  function httpSetRequestHeader(key, value) {
    return core.opSync("opEvSetRequestHeader", key, value);
  }

  function httpGetRequestUriScheme() {
    return core.opSync("opEvGetRequestUriScheme");
  }

  function httpGetRequestUriAuthority() {
    return core.opSync("opEvGetRequestUriAuthority");
  }

  function httpGetRequestUriPath() {
    return core.opSync("opEvGetRequestUriPath");
  }

  function httpGetRequestUriQuery() {
    return core.opSync("opEvGetRequestUriQuery");
  }

  function httpGetRequestUriPathQuery() {
    return core.opSync("opEvGetRequestUriPathQuery");
  }

  function httpGetRequestUriHost() {
    return core.opSync("opEvGetRequestUriHost");
  }

  function httpGetRequestUriPort() {
    return core.opSync("opEvGetRequestUriPort");
  }

  function httpGetRequestMethod() {
    return core.opSync("opEvGetRequestMethod");
  }

  function httpGetRequestVersion() {
    return core.opSync("opEvGetRequestVersion");
  }

  function httpGetRequestBodyReadStream() {
    return core.opSync("opEvGetRequestBodyReadStream");
  }

  async function httpReadRequestBodyChunk(rid, buffer) {
    return core.opAsync("opEvReadRequestBodyChunk", rid, buffer);
  }

  function httpGetRequestBodySizeHint(buffer) {
    return core.opAsync("opEvGetRequestBodySizeHint", buffer);
  }

  function httpSetResponseParts(key, value) {
    return core.opSync("opHttpSetResponseParts", key, value);
  }

  async function httpSetSendResponseBody(buf) {
    return core.opAsync("opEvSetSendResponseBody", buf);
  }

  async function httpSetSendResponseBodyWriteStream(buf) {
    return core.opAsync("opEvSetSendResponseBodyWriteStream", buf);
  }

  async function httpWriteResponseBodyChunk(rid, buf) {
    return core.opAsync("opEvWriteResponseBodyChunk", rid, buf);
  }

  window.__bootstrap.httpEvent = {
    httpGetRequestHeaders,
    httpGetRequestHeader,
    httpSetRequestHeader,
    httpGetRequestUriScheme,
    httpGetRequestUriAuthority,
    httpGetRequestUriPath,
    httpGetRequestUriQuery,
    httpGetRequestMethod,
    httpGetRequestVersion,
    httpGetRequestBodySizeHint,
    httpGetRequestBodyReadStream,
    httpReadRequestBodyChunk,
    httpSetResponseParts,
    httpSetSendResponseBody,
    httpSetSendResponseBodyWriteStream,
    httpWriteResponseBodyChunk,
    httpGetRequestUriPathQuery,
    httpGetRequestUriHost,
    httpGetRequestUriPort,
  };
})(globalThis);
