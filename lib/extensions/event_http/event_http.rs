// Copyright 2021 the Gigamono authors. All rights reserved. Apache 2.0 license.

use crate::events::Events;
use crate::permissions::events::{HttpEvent, UrlPathString};
use crate::permissions::Permissions;
use deno_core::{error::AnyError, include_js_files, op_async, Extension, OpState};
use deno_core::{op_sync, ZeroCopyBuf};
use std::cell::RefCell;
use std::convert::TryFrom;
use std::mem;
use std::rc::Rc;
use std::str::FromStr;
use utilities::errors;
use utilities::http::body::{Bytes, HttpBody};
use utilities::http::header::{HeaderName, HeaderValue};
use utilities::http::{Body, Response, StatusCode, Version};

pub fn event_http(permissions: Rc<Permissions>, events: Rc<RefCell<Events>>) -> Extension {
    let extension = Extension::builder()
        .js(include_js_files!(
            prefix "sys:ext/fs",
            "lib/extensions/fs/01_event_http.js",
        ))
        .ops(vec![
            // Request.
            ("op_get_request_header", op_sync(op_get_request_header)),
            ("op_set_request_header", op_sync(op_set_request_header)),
            (
                "op_get_request_uri_scheme",
                op_sync(op_get_request_uri_scheme),
            ),
            (
                "op_get_request_uri_authority",
                op_sync(op_get_request_uri_authority),
            ),
            ("op_get_request_uri_path", op_sync(op_get_request_uri_path)),
            (
                "op_get_request_uri_query",
                op_sync(op_get_request_uri_query),
            ),
            ("op_get_request_method", op_sync(op_get_request_method)),
            ("op_read_request_body", op_async(op_read_request_body)),
            // Response.
            ("op_set_response_header", op_sync(op_set_response_header)),
            ("op_set_response_status", op_sync(op_set_response_status)),
            ("op_set_response_version", op_sync(op_set_response_version)),
            ("op_write_response_body", op_sync(op_write_response_body)),
            ("op_send_response", op_async(op_send_response)),
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

fn op_get_request_header(state: &mut OpState, key: String, _: ()) -> Result<Vec<u8>, AnyError> {
    let events_rc = Rc::clone(state.borrow::<Rc<RefCell<Events>>>());
    let events = events_rc.borrow();

    // Get request from event.
    let (request, path) = match events.http.as_ref() {
        Some(event) => (&event.request, &event.path),
        None => return errors::missing_error_t(r#"unsupported event, "HttpEvent""#),
    };

    // Check read permission.
    let permissions = Rc::clone(state.borrow::<Rc<Permissions>>());
    permissions.check_sync(HttpEvent::ReadRequest, UrlPathString::from(path))?;

    // Get the value of request header.
    let value = match request.headers().get(&key) {
        Some(value) => value.as_ref().to_owned(),
        None => return errors::missing_error_t(format!(r#"missing header, "{:?}""#, key)),
    };

    Ok(value)
}

fn op_set_request_header(
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
    permissions.check_sync(HttpEvent::ModifyRequest, UrlPathString::from(path))?;

    // Set header.
    let optional = request
        .headers_mut()
        .insert(HeaderName::from_str(&key)?, HeaderValue::from_str(&value)?);

    Ok(optional.map(|_| value))
}

fn op_get_request_uri_scheme(
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
    permissions.check_sync(HttpEvent::ReadRequest, UrlPathString::from(path))?;

    // Get the value of request scheme.
    let authority = request.uri().scheme();

    Ok(authority.map(|v| v.to_string()))
}

fn op_get_request_uri_authority(
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
    permissions.check_sync(HttpEvent::ReadRequest, UrlPathString::from(path))?;

    // Get the value of request authority.
    let authority = request.uri().authority();

    Ok(authority.map(|v| v.to_string()))
}

fn op_get_request_uri_query(state: &mut OpState, _: (), _: ()) -> Result<Option<String>, AnyError> {
    let events_rc = Rc::clone(state.borrow::<Rc<RefCell<Events>>>());
    let events = events_rc.borrow();

    // Get request from event.
    let (request, path) = match events.http.as_ref() {
        Some(event) => (&event.request, &event.path),
        None => return errors::missing_error_t(r#"unsupported event, "HttpEvent""#),
    };

    // Check read permission.
    let permissions = Rc::clone(state.borrow::<Rc<Permissions>>());
    permissions.check_sync(HttpEvent::ReadRequest, UrlPathString::from(path))?;

    // Get the value of request query.
    let query = request.uri().query();

    Ok(query.map(|v| v.to_owned()))
}

fn op_get_request_uri_path(state: &mut OpState, _: (), _: ()) -> Result<String, AnyError> {
    let events_rc = Rc::clone(state.borrow::<Rc<RefCell<Events>>>());
    let events = events_rc.borrow();

    // Get request from event.
    let (request, path) = match events.http.as_ref() {
        Some(event) => (&event.request, &event.path),
        None => return errors::missing_error_t(r#"unsupported event, "HttpEvent""#),
    };

    // Check read permission.
    let permissions = Rc::clone(state.borrow::<Rc<Permissions>>());
    permissions.check_sync(HttpEvent::ReadRequest, UrlPathString::from(path))?;

    // Get the value of request path.
    let uri_path = request.uri().path();

    Ok(uri_path.to_owned())
}

fn op_get_request_method(state: &mut OpState, _: (), _: ()) -> Result<String, AnyError> {
    let events_rc = Rc::clone(state.borrow::<Rc<RefCell<Events>>>());
    let events = events_rc.borrow();

    // Get request from event.
    let (request, path) = match events.http.as_ref() {
        Some(event) => (&event.request, &event.path),
        None => return errors::missing_error_t(r#"unsupported event, "HttpEvent""#),
    };

    // Check read permission.
    let permissions = Rc::clone(state.borrow::<Rc<Permissions>>());
    permissions.check_sync(HttpEvent::ReadRequest, UrlPathString::from(path))?;

    // Get the value of request method.
    let method = request.method().to_string();

    Ok(method)
}

async fn op_read_request_body(
    state: Rc<RefCell<OpState>>,
    mut buf: ZeroCopyBuf,
    _: (),
) -> Result<usize, AnyError> {
    let events_rc = Rc::clone(state.borrow().borrow::<Rc<RefCell<Events>>>());
    let mut events = events_rc.borrow_mut();

    // Get request from event.
    let request = match events.http.as_mut() {
        Some(event) => &mut event.request,
        None => return errors::missing_error_t(r#"unsupported event, "HttpEvent""#),
    };

    // Check read permission.
    let permissions = Rc::clone(state.borrow().borrow::<Rc<Permissions>>());
    permissions.check_sync(
        HttpEvent::ReadRequest,
        UrlPathString::from(request.uri().path()),
    )?;

    // Get the next buffer from body.
    let body = request.body_mut();
    futures_util::pin_mut!(body);

    // Copy body bytes into buffer.
    let bytes = body.data().await.unwrap_or(Ok(Bytes::new()))?;
    buf.copy_from_slice(bytes.as_ref());

    Ok(bytes.len())
}

fn op_set_response_header(
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
    permissions.check_sync(HttpEvent::WriteResponse, UrlPathString::from(path))?;

    // Set header.
    let optional = response
        .headers_mut()
        .insert(HeaderName::from_str(&key)?, HeaderValue::from_str(&value)?);

    Ok(optional.map(|_| value))
}

fn op_set_response_status(state: &mut OpState, status: u16, _: ()) -> Result<(), AnyError> {
    let events_rc = Rc::clone(state.borrow::<Rc<RefCell<Events>>>());
    let mut events = events_rc.borrow_mut();

    // Get objects from http.event.
    let (response, path) = match events.http.as_mut() {
        Some(event) => (&mut event.response, &event.path),
        None => return errors::missing_error_t(r#"unsupported event, "HttpEvent""#),
    };

    // Check read permission.
    let permissions = Rc::clone(state.borrow::<Rc<Permissions>>());
    permissions.check_sync(HttpEvent::WriteResponse, UrlPathString::from(path))?;

    // Write status.
    *response.status_mut() = StatusCode::try_from(status)?;

    Ok(())
}

fn op_set_response_version(state: &mut OpState, version: String, _: ()) -> Result<(), AnyError> {
    let events_rc = Rc::clone(state.borrow::<Rc<RefCell<Events>>>());
    let mut events = events_rc.borrow_mut();

    // Get objects from http.event.
    let (response, path) = match events.http.as_mut() {
        Some(event) => (&mut event.response, &event.path),
        None => return errors::missing_error_t(r#"unsupported event, "HttpEvent""#),
    };

    // Check read permission.
    let permissions = Rc::clone(state.borrow::<Rc<Permissions>>());
    permissions.check_sync(HttpEvent::WriteResponse, UrlPathString::from(path))?;

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

fn op_write_response_body(state: &mut OpState, buf: Vec<u8>, _: ()) -> Result<(), AnyError> {
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
    permissions.check_sync(HttpEvent::WriteResponse, UrlPathString::from(path))?;

    // Write to body.
    *response.body_mut() = Body::from(buf);

    Ok(())
}

async fn op_send_response(state: Rc<RefCell<OpState>>, _: (), _: ()) -> Result<(), AnyError> {
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
    permissions.check_sync(HttpEvent::SendResponse, UrlPathString::from(path))?;

    // Send response.
    responder.send_response(response).await?;

    Ok(())
}
