// Copyright 2021 the Gigamono authors. All rights reserved. Apache 2.0 license.

use crate::events::Events;
use crate::permissions::events::event_http::{HttpEvent, Path};
use crate::permissions::Permissions;
use deno_core::{error::AnyError, include_js_files, op_async, Extension, OpState};
use deno_core::{op_sync, Resource, ResourceId, ZeroCopyBuf};
use futures_util::Stream;
use std::cell::RefCell;
use std::convert::TryFrom;
use std::mem;
use std::pin::Pin;
use std::rc::Rc;
use std::str::FromStr;
use std::task::{Context, Poll};
use tokio::io::AsyncReadExt;
use tokio_util::io::StreamReader;
use utilities::errors;
use utilities::http::body::{Bytes, HttpBody};
use utilities::http::header::{HeaderName, HeaderValue};
use utilities::http::{Body, Response, StatusCode, Version};

pub fn event_http(permissions: Rc<Permissions>, events: Rc<RefCell<Events>>) -> Extension {
    let extension = Extension::builder()
        .js(include_js_files!(
            prefix "(tera:extensions) ",
            "lib/extensions/event_http/01_event_http.js",
        ))
        .ops(vec![
            // Request.
            (
                "op_ev_get_request_header",
                op_sync(op_ev_get_request_header),
            ),
            (
                "op_ev_set_request_header",
                op_sync(op_ev_set_request_header),
            ),
            (
                "op_ev_get_request_uri_scheme",
                op_sync(op_ev_get_request_uri_scheme),
            ),
            (
                "op_ev_get_request_uri_authority",
                op_sync(op_ev_get_request_uri_authority),
            ),
            (
                "op_ev_get_request_uri_path",
                op_sync(op_ev_get_request_uri_path),
            ),
            (
                "op_ev_get_request_uri_query",
                op_sync(op_ev_get_request_uri_query),
            ),
            (
                "op_ev_get_request_method",
                op_sync(op_ev_get_request_method),
            ),
            (
                "op_ev_get_request_body_size_hint",
                op_sync(op_ev_get_request_body_size_hint),
            ),
            (
                "op_ev_get_request_body_stream",
                op_sync(op_ev_get_request_body_stream),
            ),
            (
                "op_ev_read_request_body_chunk",
                op_async(op_ev_read_request_body_chunk),
            ),
            // Response.
            (
                "op_ev_set_response_header",
                op_sync(op_ev_set_response_header),
            ),
            (
                "op_ev_set_response_status",
                op_sync(op_ev_set_response_status),
            ),
            (
                "op_ev_set_response_version",
                op_sync(op_ev_set_response_version),
            ),
            (
                "op_ev_write_response_body",
                op_sync(op_ev_write_response_body),
            ),
            ("op_ev_send_response", op_async(op_ev_send_response)),
        ])
        .state(move |state| {
            if !state.has::<Permissions>() {
                state.put(Rc::clone(&permissions));
            }

            if !state.has::<Events>() {
                state.put(Rc::clone(&events));
            }

            Ok(())
        })
        .build();

    extension
}

struct BodyStream(Body);

impl Stream for BodyStream {
    type Item = Result<Bytes, std::io::Error>;

    fn poll_next(mut self: Pin<&mut Self>, cx: &mut Context) -> Poll<Option<Self::Item>> {
        match futures_core::ready!(Pin::new(&mut self.0).poll_next(cx)) {
            Some(Ok(chunk)) => Poll::Ready(Some(Ok(chunk))),
            Some(Err(err)) => Poll::Ready(Some(Err(std::io::Error::new(
                std::io::ErrorKind::Other,
                err,
            )))),
            None => Poll::Ready(None),
        }
    }
}

struct StreamReaderResource(RefCell<StreamReader<BodyStream, Bytes>>);

impl Resource for StreamReaderResource {}

fn op_ev_get_request_header(state: &mut OpState, key: String, _: ()) -> Result<Vec<u8>, AnyError> {
    let events_rc = Rc::clone(state.borrow::<Rc<RefCell<Events>>>());
    let events = events_rc.borrow();

    // Get request from event.
    let (request, path) = match events.http.as_ref() {
        Some(event) => (&event.request, &event.path),
        None => return errors::missing_error_t(r#"unsupported event, "HttpEvent""#),
    };

    // Check read permission.
    let permissions = Rc::clone(state.borrow::<Rc<Permissions>>());
    permissions.check(HttpEvent::ReadRequest, Path::from(path))?;

    // Get the value of request header.
    let value = match request.headers().get(&key) {
        Some(value) => value.as_ref().to_owned(),
        None => return errors::missing_error_t(format!(r#"missing header, "{:?}""#, key)),
    };

    Ok(value)
}

fn op_ev_set_request_header(
    state: &mut OpState,
    key: String,
    value: String,
) -> Result<Option<String>, AnyError> {
    let events_rc = Rc::clone(state.borrow::<Rc<RefCell<Events>>>());
    let mut events = events_rc.borrow_mut();

    // Get request from event.
    let (request, path) = match events.http.as_mut() {
        Some(event) => (&mut event.request, &event.path),
        None => return errors::missing_error_t(r#"unsupported event, "HttpEvent""#),
    };

    // Check modify permission.
    let permissions = Rc::clone(state.borrow::<Rc<Permissions>>());
    permissions.check(HttpEvent::ModifyRequest, Path::from(path))?;

    // Set header.
    let optional = request
        .headers_mut()
        .insert(HeaderName::from_str(&key)?, HeaderValue::from_str(&value)?);

    Ok(optional.map(|_| value))
}

fn op_ev_get_request_uri_scheme(
    state: &mut OpState,
    _: (),
    _: (),
) -> Result<Option<String>, AnyError> {
    let events_rc = Rc::clone(state.borrow::<Rc<RefCell<Events>>>());
    let events = events_rc.borrow();

    // Get request from event.
    let (request, path) = match events.http.as_ref() {
        Some(event) => (&event.request, &event.path),
        None => return errors::missing_error_t(r#"unsupported event, "HttpEvent""#),
    };

    // Check read permission.
    let permissions = Rc::clone(state.borrow::<Rc<Permissions>>());
    permissions.check(HttpEvent::ReadRequest, Path::from(path))?;

    // Get the value of request scheme.
    let authority = request.uri().scheme();

    Ok(authority.map(|v| v.to_string()))
}

fn op_ev_get_request_uri_authority(
    state: &mut OpState,
    _: (),
    _: (),
) -> Result<Option<String>, AnyError> {
    let events_rc = Rc::clone(state.borrow::<Rc<RefCell<Events>>>());
    let events = events_rc.borrow();

    // Get request from event.
    let (request, path) = match events.http.as_ref() {
        Some(event) => (&event.request, &event.path),
        None => return errors::missing_error_t(r#"unsupported event, "HttpEvent""#),
    };

    // Check read permission.
    let permissions = Rc::clone(state.borrow::<Rc<Permissions>>());
    permissions.check(HttpEvent::ReadRequest, Path::from(path))?;

    // Get the value of request authority.
    let authority = request.uri().authority();

    Ok(authority.map(|v| v.to_string()))
}

fn op_ev_get_request_uri_query(
    state: &mut OpState,
    _: (),
    _: (),
) -> Result<Option<String>, AnyError> {
    let events_rc = Rc::clone(state.borrow::<Rc<RefCell<Events>>>());
    let events = events_rc.borrow();

    // Get request from event.
    let (request, path) = match events.http.as_ref() {
        Some(event) => (&event.request, &event.path),
        None => return errors::missing_error_t(r#"unsupported event, "HttpEvent""#),
    };

    // Check read permission.
    let permissions = Rc::clone(state.borrow::<Rc<Permissions>>());
    permissions.check(HttpEvent::ReadRequest, Path::from(path))?;

    // Get the value of request query.
    let query = request.uri().query();

    Ok(query.map(|v| v.to_owned()))
}

fn op_ev_get_request_uri_path(state: &mut OpState, _: (), _: ()) -> Result<String, AnyError> {
    let events_rc = Rc::clone(state.borrow::<Rc<RefCell<Events>>>());
    let events = events_rc.borrow();

    // Get request from event.
    let (request, path) = match events.http.as_ref() {
        Some(event) => (&event.request, &event.path),
        None => return errors::missing_error_t(r#"unsupported event, "HttpEvent""#),
    };

    // Check read permission.
    let permissions = Rc::clone(state.borrow::<Rc<Permissions>>());
    permissions.check(HttpEvent::ReadRequest, Path::from(path))?;

    // Get the value of request path.
    let uri_path = request.uri().path();

    Ok(uri_path.to_owned())
}

fn op_ev_get_request_method(state: &mut OpState, _: (), _: ()) -> Result<String, AnyError> {
    let events_rc = Rc::clone(state.borrow::<Rc<RefCell<Events>>>());
    let events = events_rc.borrow();

    // Get request from event.
    let (request, path) = match events.http.as_ref() {
        Some(event) => (&event.request, &event.path),
        None => return errors::missing_error_t(r#"unsupported event, "HttpEvent""#),
    };

    // Check read permission.
    let permissions = Rc::clone(state.borrow::<Rc<Permissions>>());
    permissions.check(HttpEvent::ReadRequest, Path::from(path))?;

    // Get the value of request method.
    let method = request.method().to_string();

    Ok(method)
}

fn op_ev_get_request_body_size_hint(state: &mut OpState, _: (), _: ()) -> Result<u64, AnyError> {
    let events_rc = Rc::clone(state.borrow::<Rc<RefCell<Events>>>());
    let events = events_rc.borrow();

    // Get request from event.
    let (request, path) = match events.http.as_ref() {
        Some(event) => (&event.request, &event.path),
        None => return errors::missing_error_t(r#"unsupported event, "HttpEvent""#),
    };

    // Check read permission.
    let permissions = Rc::clone(state.borrow::<Rc<Permissions>>());
    permissions.check(HttpEvent::ReadRequest, Path::from(path))?;

    // Get the value of request body size.
    let size = HttpBody::size_hint(request.body())
        .exact()
        .unwrap_or_default();

    Ok(size)
}

fn op_ev_get_request_body_stream(state: &mut OpState, _: (), _: ()) -> Result<u32, AnyError> {
    let events_rc = Rc::clone(state.borrow::<Rc<RefCell<Events>>>());
    let mut events = events_rc.borrow_mut();

    // Get request from event.
    let (request, path) = match events.http.as_mut() {
        Some(event) => (&mut event.request, &event.path),
        None => return errors::missing_error_t(r#"unsupported event, "HttpEvent""#),
    };

    // Check read permission.
    let permissions = Rc::clone(state.borrow::<Rc<Permissions>>());
    permissions.check(HttpEvent::ReadRequest, Path::from(path))?;

    // Take ownership of body.
    let body = mem::replace(request.body_mut(), Body::empty());
    let reader = StreamReader::new(BodyStream(body));

    // Add stream reader to resource table.
    let rid = state
        .resource_table
        .add(StreamReaderResource(RefCell::new(reader)));

    Ok(rid)
}

async fn op_ev_read_request_body_chunk(
    state: Rc<RefCell<OpState>>,
    rid: ResourceId,
    mut buf: ZeroCopyBuf,
) -> Result<usize, AnyError> {
    let events_rc = Rc::clone(state.borrow().borrow::<Rc<RefCell<Events>>>());
    let events = events_rc.borrow_mut();

    // Get request from event.
    let path = match events.http.as_ref() {
        Some(event) => &event.path,
        None => return errors::missing_error_t(r#"unsupported event, "HttpEvent""#),
    };

    // Check read permission.
    let permissions = Rc::clone(state.borrow().borrow::<Rc<Permissions>>());
    permissions.check(HttpEvent::ReadRequest, Path::from(path))?;

    // Get the next buffer from body.
    let reader = state
        .borrow()
        .resource_table
        .get::<StreamReaderResource>(rid)?;

    let total_read = reader.0.borrow_mut().read(&mut buf).await?;

    Ok(total_read)
}

fn op_ev_set_response_header(
    state: &mut OpState,
    key: String,
    value: String,
) -> Result<Option<String>, AnyError> {
    let events_rc = Rc::clone(state.borrow::<Rc<RefCell<Events>>>());
    let mut events = events_rc.borrow_mut();

    // Get objects from http.event.
    let (response, path) = match events.http.as_mut() {
        Some(event) => (&mut event.response, &event.path),
        None => return errors::missing_error_t(r#"unsupported event, "HttpEvent""#),
    };

    // Check read permission.
    let permissions = Rc::clone(state.borrow::<Rc<Permissions>>());
    permissions.check(HttpEvent::WriteResponse, Path::from(path))?;

    // Set header.
    let optional = response
        .headers_mut()
        .insert(HeaderName::from_str(&key)?, HeaderValue::from_str(&value)?);

    Ok(optional.map(|_| value))
}

fn op_ev_set_response_status(state: &mut OpState, status: u16, _: ()) -> Result<(), AnyError> {
    let events_rc = Rc::clone(state.borrow::<Rc<RefCell<Events>>>());
    let mut events = events_rc.borrow_mut();

    // Get objects from http.event.
    let (response, path) = match events.http.as_mut() {
        Some(event) => (&mut event.response, &event.path),
        None => return errors::missing_error_t(r#"unsupported event, "HttpEvent""#),
    };

    // Check read permission.
    let permissions = Rc::clone(state.borrow::<Rc<Permissions>>());
    permissions.check(HttpEvent::WriteResponse, Path::from(path))?;

    // Write status.
    *response.status_mut() = StatusCode::try_from(status)?;

    Ok(())
}

fn op_ev_set_response_version(state: &mut OpState, version: String, _: ()) -> Result<(), AnyError> {
    let events_rc = Rc::clone(state.borrow::<Rc<RefCell<Events>>>());
    let mut events = events_rc.borrow_mut();

    // Get objects from http.event.
    let (response, path) = match events.http.as_mut() {
        Some(event) => (&mut event.response, &event.path),
        None => return errors::missing_error_t(r#"unsupported event, "HttpEvent""#),
    };

    // Check read permission.
    let permissions = Rc::clone(state.borrow::<Rc<Permissions>>());
    permissions.check(HttpEvent::WriteResponse, Path::from(path))?;

    // Write version.
    *response.version_mut() = match version.as_str() {
        "0.9" => Version::HTTP_09,
        "1.0" => Version::HTTP_10,
        "1.1" => Version::HTTP_11,
        "2" => Version::HTTP_2,
        "3" => Version::HTTP_3,
        _ => {
            return errors::type_error_t(format!(
                r#"invalid HTTP version, "{}". Can be one of ["0.9", "1.0", "1.1", "2", "3"]"#,
                version
            ))
        }
    };

    Ok(())
}

fn op_ev_write_response_body(state: &mut OpState, buf: ZeroCopyBuf, _: ()) -> Result<(), AnyError> {
    // TODO(appcypher): Support body streaming.
    let events_rc = Rc::clone(state.borrow::<Rc<RefCell<Events>>>());
    let mut events = events_rc.borrow_mut();

    // Get objects from http.event.
    let (response, path) = match events.http.as_mut() {
        Some(event) => (&mut event.response, &event.path),
        None => return errors::missing_error_t(r#"unsupported event, "HttpEvent""#),
    };

    // Check read permission.
    let permissions = Rc::clone(state.borrow::<Rc<Permissions>>());
    permissions.check(HttpEvent::WriteResponse, Path::from(path))?;

    // Write to body.
    *response.body_mut() = Body::from(buf.to_vec());

    Ok(())
}

async fn op_ev_send_response(state: Rc<RefCell<OpState>>, _: (), _: ()) -> Result<(), AnyError> {
    let events_rc = Rc::clone(state.borrow().borrow::<Rc<RefCell<Events>>>());
    let mut events = events_rc.borrow_mut();

    // Get request from event.
    let (response, responder, path) = match events.http.as_mut() {
        Some(event) => (
            mem::replace(&mut event.response, Response::default()), // Take ownership of response.
            &mut event.responder,
            &event.path,
        ),
        None => return errors::missing_error_t(r#"unsupported event, "HttpEvent""#),
    };

    // Check read permission.
    let permissions = Rc::clone(state.borrow().borrow::<Rc<Permissions>>());
    permissions.check(HttpEvent::SendResponse, Path::from(path))?;

    // Send response.
    responder.send_response(response).await?;

    Ok(())
}
