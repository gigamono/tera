// Copyright 2021 the Gigamono authors. All rights reserved. Apache 2.0 license.
//! No support for non-ascii headers yet.

use crate::events::Events;
use crate::permissions::events::event_http::{HttpEvent, HttpEventPath};
use crate::permissions::Permissions;
use deno_core::parking_lot::Mutex;
use deno_core::{error::AnyError, include_js_files, op_async, Extension, OpState};
use deno_core::{op_sync, Resource, ResourceId, ZeroCopyBuf};
use futures_util::Stream;
use serde::Deserialize;
use std::cell::RefCell;
use std::collections::{HashMap, VecDeque};
use std::convert::TryFrom;
use std::mem;
use std::pin::Pin;
use std::rc::Rc;
use std::str::FromStr;
use std::sync::Arc;
use std::task::{Context, Poll, Waker};
use tokio::io::AsyncReadExt;
use tokio_util::io::StreamReader;
use utilities::errors;
use utilities::hyper::body::{Bytes, HttpBody};
use utilities::hyper::header::{HeaderName, HeaderValue};
use utilities::hyper::{Body, HeaderMap, StatusCode, Version};

pub fn event_http(permissions: Rc<RefCell<Permissions>>, events: Rc<RefCell<Events>>) -> Extension {
    let extension = Extension::builder()
        .js(include_js_files!(
            prefix "(tera:extensions) ",
            "lib/extensions/event_http/01_event_http.js",
        ))
        .ops(vec![
            // Request.
            (
                "opEvGetRequestHeaders",
                op_sync(op_http_get_request_headers),
            ),
            ("opEvGetRequestHeader", op_sync(op_http_get_request_header)),
            ("opEvSetRequestHeader", op_sync(op_http_set_request_header)),
            (
                "opEvGetRequestUriScheme",
                op_sync(op_http_get_request_uri_scheme),
            ),
            (
                "opEvGetRequestUriAuthority",
                op_sync(op_http_get_request_uri_authority),
            ),
            (
                "opEvGetRequestUriPath",
                op_sync(op_http_get_request_uri_path),
            ),
            (
                "opEvGetRequestUriQuery",
                op_sync(op_http_get_request_uri_query),
            ),
            (
                "opEvGetRequestUriPathQuery",
                op_sync(op_http_get_request_uri_path_query),
            ),
            (
                "opEvGetRequestUriHost",
                op_sync(op_http_get_request_uri_host),
            ),
            (
                "opEvGetRequestUriPort",
                op_sync(op_http_get_request_uri_port),
            ),
            ("opEvGetRequestMethod", op_sync(op_http_get_request_method)),
            (
                "opEvGetRequestVersion",
                op_sync(op_http_get_request_version),
            ),
            (
                "opEvGetRequestBodySizeHint",
                op_sync(op_http_get_request_body_size_hint),
            ),
            (
                "opEvGetRequestBodyReadStream",
                op_sync(op_http_get_request_body_read_stream),
            ),
            (
                "opEvReadRequestBodyChunk",
                op_async(op_http_read_request_body_chunk),
            ),
            // Response.
            (
                "opHttpSetResponseParts",
                op_sync(op_http_set_response_parts),
            ),
            (
                "opEvSetSendResponseBody",
                op_async(op_http_set_send_response_body),
            ),
            (
                "opEvSetSendResponseBodyWriteStream",
                op_async(op_http_set_send_response_body_write_stream),
            ),
            (
                "opEvWriteResponseBodyChunk",
                op_async(op_http_write_response_body_chunk),
            ),
        ])
        .state(move |state| {
            if !state.has::<Rc<RefCell<Permissions>>>() {
                state.put(Rc::clone(&permissions));
            }

            if !state.has::<Rc<RefCell<Events>>>() {
                state.put(Rc::clone(&events));
            }

            Ok(())
        })
        .build();

    extension
}

// Each buffer takes 16KB (check SIZE_PER_ITER in lib/runtime/postscripts/01_common.js). The queue is roughly 16MB.
const MAX_BUFFER_QUEUE_SIZE: usize = 1024;

#[derive(Deserialize, Default, Debug)]
pub struct ResponseParts {
    pub status: u16,
    pub version: String,
    pub headers: HashMap<String, String>,
}

// This is where reponse buffer streams are stored.
// Using Arc<Mutex<T>> here cause Body::wrap_stream requires Send stream.
#[derive(Default)]
struct BufferQueue {
    pub queue: VecDeque<Vec<u8>>,
    waker: Option<Waker>, // Used to wake associated stream future when the queue is no longer empty.
}

type BufferQueueShared = Arc<Mutex<BufferQueue>>;

struct BodyReadStream(Body);

struct StreamReaderResource(RefCell<StreamReader<BodyReadStream, Bytes>>);

struct BodyWriteStream(BufferQueueShared);

struct BufferQueueResource(BufferQueueShared);

impl BufferQueue {
    pub fn new() -> Self {
        Self {
            queue: VecDeque::new(),
            waker: None,
        }
    }
}

impl Stream for BodyReadStream {
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

impl Stream for BodyWriteStream {
    type Item = Result<Vec<u8>, std::io::Error>;

    fn poll_next(self: Pin<&mut Self>, cx: &mut Context) -> Poll<Option<Self::Item>> {
        // An empty buffer signifies closing a stream.
        let buffer = self.0.lock().queue.pop_front();
        match buffer {
            Some(buffer) => {
                if buffer.len() > 0 {
                    Poll::Ready(Some(Ok(buffer)))
                } else {
                    Poll::Ready(None) // End stream.
                }
            }
            None => {
                self.0.lock().waker = Some(cx.waker().clone()); // Does not reach here
                Poll::Pending
            }
        }
    }
}

impl Resource for StreamReaderResource {}

impl Resource for BufferQueueResource {}

fn op_http_get_request_headers(
    state: &mut OpState,
    _: (),
    _: (),
) -> Result<HashMap<String, String>, AnyError> {
    let events_rc = Rc::clone(state.borrow::<Rc<RefCell<Events>>>());
    let events = events_rc.borrow();

    // Get request from event.
    let (request, path) = match events.http.as_ref() {
        Some(event) => (&event.request, &event.path),
        None => return errors::missing_error_t(r#"unsupported event, "HttpEvent""#),
    };

    // Check read permission.
    let permissions = Rc::clone(state.borrow::<Rc<RefCell<Permissions>>>());
    permissions
        .borrow()
        .check(HttpEvent::ReadRequest, HttpEventPath::from(path))?;

    // Populate a hashmap from header key-value pair.
    let mut map = HashMap::new();
    for (k, v) in request.headers().iter() {
        let k = k.as_str().to_owned();
        let v = v.to_str()?.to_owned();
        map.insert(k, v);
    }

    Ok(map)
}

fn op_http_get_request_header(state: &mut OpState, key: String, _: ()) -> Result<String, AnyError> {
    let events_rc = Rc::clone(state.borrow::<Rc<RefCell<Events>>>());
    let events = events_rc.borrow();

    // Get request from event.
    let (request, path) = match events.http.as_ref() {
        Some(event) => (&event.request, &event.path),
        None => return errors::missing_error_t(r#"unsupported event, "HttpEvent""#),
    };

    // Check read permission.
    let permissions = Rc::clone(state.borrow::<Rc<RefCell<Permissions>>>());
    permissions
        .borrow()
        .check(HttpEvent::ReadRequest, HttpEventPath::from(path))?;

    // Get the value of request header.
    let value = match request.headers().get(&key) {
        Some(value) => value.to_str()?.to_owned(),
        None => return errors::missing_error_t(format!(r#"missing header, "{:?}""#, key)),
    };

    Ok(value)
}

fn op_http_set_request_header(
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
    let permissions = Rc::clone(state.borrow::<Rc<RefCell<Permissions>>>());
    permissions
        .borrow()
        .check(HttpEvent::ModifyRequest, HttpEventPath::from(path))?;

    // Set header.
    let optional = request
        .headers_mut()
        .insert(HeaderName::from_str(&key)?, HeaderValue::from_str(&value)?);

    Ok(optional.map(|_| value))
}

fn op_http_get_request_uri_scheme(
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
    let permissions = Rc::clone(state.borrow::<Rc<RefCell<Permissions>>>());
    permissions
        .borrow()
        .check(HttpEvent::ReadRequest, HttpEventPath::from(path))?;

    // Get the value of request scheme.
    let authority = request.uri().scheme();

    Ok(authority.map(|v| v.to_string()))
}

fn op_http_get_request_uri_authority(
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
    let permissions = Rc::clone(state.borrow::<Rc<RefCell<Permissions>>>());
    permissions
        .borrow()
        .check(HttpEvent::ReadRequest, HttpEventPath::from(path))?;

    // Get the value of request authority.
    let authority = request.uri().authority();

    Ok(authority.map(|v| v.to_string()))
}

fn op_http_get_request_uri_query(
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
    let permissions = Rc::clone(state.borrow::<Rc<RefCell<Permissions>>>());
    permissions
        .borrow()
        .check(HttpEvent::ReadRequest, HttpEventPath::from(path))?;

    // Get the value of request query.
    let query = request.uri().query();

    Ok(query.map(|v| v.to_owned()))
}

fn op_http_get_request_uri_path(state: &mut OpState, _: (), _: ()) -> Result<String, AnyError> {
    let events_rc = Rc::clone(state.borrow::<Rc<RefCell<Events>>>());
    let events = events_rc.borrow();

    // Get request from event.
    let (request, path) = match events.http.as_ref() {
        Some(event) => (&event.request, &event.path),
        None => return errors::missing_error_t(r#"unsupported event, "HttpEvent""#),
    };

    // Check read permission.
    let permissions = Rc::clone(state.borrow::<Rc<RefCell<Permissions>>>());
    permissions
        .borrow()
        .check(HttpEvent::ReadRequest, HttpEventPath::from(path))?;

    // Get the value of request path.
    let uri_path = request.uri().path();

    Ok(uri_path.to_owned())
}

fn op_http_get_request_uri_path_query(
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
    let permissions = Rc::clone(state.borrow::<Rc<RefCell<Permissions>>>());
    permissions
        .borrow()
        .check(HttpEvent::ReadRequest, HttpEventPath::from(path))?;

    // Get the value of request path and query.
    let query = request.uri().path_and_query();

    Ok(query.map(|v| v.to_string()))
}

fn op_http_get_request_uri_host(
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
    let permissions = Rc::clone(state.borrow::<Rc<RefCell<Permissions>>>());
    permissions
        .borrow()
        .check(HttpEvent::ReadRequest, HttpEventPath::from(path))?;

    // Get the value of request host.
    let query = request.uri().host();

    Ok(query.map(|v| v.to_string()))
}

fn op_http_get_request_uri_port(
    state: &mut OpState,
    _: (),
    _: (),
) -> Result<Option<u16>, AnyError> {
    let events_rc = Rc::clone(state.borrow::<Rc<RefCell<Events>>>());
    let events = events_rc.borrow();

    // Get request from event.
    let (request, path) = match events.http.as_ref() {
        Some(event) => (&event.request, &event.path),
        None => return errors::missing_error_t(r#"unsupported event, "HttpEvent""#),
    };

    // Check read permission.
    let permissions = Rc::clone(state.borrow::<Rc<RefCell<Permissions>>>());
    permissions
        .borrow()
        .check(HttpEvent::ReadRequest, HttpEventPath::from(path))?;

    // Get the value of request port.
    Ok(request.uri().port_u16())
}

fn op_http_get_request_method(state: &mut OpState, _: (), _: ()) -> Result<String, AnyError> {
    let events_rc = Rc::clone(state.borrow::<Rc<RefCell<Events>>>());
    let events = events_rc.borrow();

    // Get request from event.
    let (request, path) = match events.http.as_ref() {
        Some(event) => (&event.request, &event.path),
        None => return errors::missing_error_t(r#"unsupported event, "HttpEvent""#),
    };

    // Check read permission.
    let permissions = Rc::clone(state.borrow::<Rc<RefCell<Permissions>>>());
    permissions
        .borrow()
        .check(HttpEvent::ReadRequest, HttpEventPath::from(path))?;

    // Get the value of request method.
    let method = request.method().to_string();

    Ok(method)
}

fn op_http_get_request_version(state: &mut OpState, _: (), _: ()) -> Result<String, AnyError> {
    let events_rc = Rc::clone(state.borrow::<Rc<RefCell<Events>>>());
    let events = events_rc.borrow();

    // Get request from event.
    let (request, path) = match events.http.as_ref() {
        Some(event) => (&event.request, &event.path),
        None => return errors::missing_error_t(r#"unsupported event, "HttpEvent""#),
    };

    // Check read permission.
    let permissions = Rc::clone(state.borrow::<Rc<RefCell<Permissions>>>());
    permissions
        .borrow()
        .check(HttpEvent::ReadRequest, HttpEventPath::from(path))?;

    // Get the value of request method.
    let version = format!("{:?}", request.version());

    Ok(version)
}

fn op_http_get_request_body_size_hint(state: &mut OpState, _: (), _: ()) -> Result<u64, AnyError> {
    let events_rc = Rc::clone(state.borrow::<Rc<RefCell<Events>>>());
    let events = events_rc.borrow();

    // Get request from event.
    let (request, path) = match events.http.as_ref() {
        Some(event) => (&event.request, &event.path),
        None => return errors::missing_error_t(r#"unsupported event, "HttpEvent""#),
    };

    // Check read permission.
    let permissions = Rc::clone(state.borrow::<Rc<RefCell<Permissions>>>());
    permissions
        .borrow()
        .check(HttpEvent::ReadRequest, HttpEventPath::from(path))?;

    // Get the value of request body size.
    let size = HttpBody::size_hint(request.body())
        .exact()
        .unwrap_or_default();

    Ok(size)
}

fn op_http_get_request_body_read_stream(
    state: &mut OpState,
    _: (),
    _: (),
) -> Result<u32, AnyError> {
    let events_rc = Rc::clone(state.borrow::<Rc<RefCell<Events>>>());
    let mut events = events_rc.borrow_mut();

    // Get request from event.
    let (request, path) = match events.http.as_mut() {
        Some(event) => (&mut event.request, &event.path),
        None => return errors::missing_error_t(r#"unsupported event, "HttpEvent""#),
    };

    // Check read permission.
    let permissions = Rc::clone(state.borrow::<Rc<RefCell<Permissions>>>());
    permissions
        .borrow()
        .check(HttpEvent::ReadRequest, HttpEventPath::from(path))?;

    // Take ownership of body.
    let body = mem::take(request.body_mut());

    // Add stream reader to resource table.
    let rid = state
        .resource_table
        .add(StreamReaderResource(RefCell::new(StreamReader::new(
            BodyReadStream(body),
        ))));

    Ok(rid)
}

async fn op_http_read_request_body_chunk(
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
    let permissions = Rc::clone(state.borrow().borrow::<Rc<RefCell<Permissions>>>());
    permissions
        .borrow()
        .check(HttpEvent::ReadRequest, HttpEventPath::from(path))?;

    // Get the next buffer from body.
    let reader = state
        .borrow()
        .resource_table
        .get::<StreamReaderResource>(rid)?;

    let total_read = reader.0.borrow_mut().read(&mut buf).await?;

    Ok(total_read)
}

fn op_http_set_response_parts(
    state: &mut OpState,
    parts: ResponseParts,
    _: (),
) -> Result<(), AnyError> {
    let events_rc = Rc::clone(state.borrow::<Rc<RefCell<Events>>>());
    let mut events = events_rc.borrow_mut();

    // Get objects from http.event.
    let (response, path) = match events.http.as_mut() {
        Some(event) => (&mut event.response, &event.path),
        None => return errors::missing_error_t(r#"unsupported event, "HttpEvent""#),
    };

    // Check read permission.
    let permissions = Rc::clone(state.borrow::<Rc<RefCell<Permissions>>>());
    permissions
        .borrow()
        .check(HttpEvent::WriteResponse, HttpEventPath::from(path))?;

    // Set headers.
    let mut map = HeaderMap::new();
    for (k, v) in parts.headers.iter() {
        map.insert(HeaderName::from_str(&k)?, HeaderValue::from_str(&v)?);
    }
    *response.headers_mut() = map;

    // Set version.
    *response.version_mut() = match parts.version.as_str() {
        "0.9" => Version::HTTP_09,
        "1.0" => Version::HTTP_10,
        "1.1" => Version::HTTP_11,
        "2" => Version::HTTP_2,
        "3" => Version::HTTP_3,
        _ => {
            return errors::type_error_t(format!(
                r#"invalid HTTP version, "{}". Can be one of ["0.9", "1.0", "1.1", "2", "3"]"#,
                parts.version
            ))
        }
    };

    // Set status.
    *response.status_mut() = StatusCode::try_from(parts.status)?;

    Ok(())
}

async fn op_http_set_send_response_body(
    state: Rc<RefCell<OpState>>,
    buf: ZeroCopyBuf,
    _: (),
) -> Result<(), AnyError> {
    let events_rc = Rc::clone(state.borrow().borrow::<Rc<RefCell<Events>>>());
    let mut events = events_rc.borrow_mut();

    // Get objects from http.event.
    let (mut response, responder, path) = match events.http.as_mut() {
        Some(event) => (
            mem::take(&mut event.response), // Take ownership of response.
            &mut event.responder,
            &event.path,
        ),
        None => return errors::missing_error_t(r#"unsupported event, "HttpEvent""#),
    };

    // Check read permission.
    let permissions = Rc::clone(state.borrow().borrow::<Rc<RefCell<Permissions>>>());
    permissions
        .borrow()
        .check(HttpEvent::WriteResponse, HttpEventPath::from(path))?;

    // Write to body if buffer is not empty.
    if buf.len() > 0 {
        *response.body_mut() = Body::from(buf.to_vec());
    }

    // Send response.
    responder.send_response(response).await?;

    Ok(())
}

// As there is no way for ops to call js code which would enable lazy streaming.
// We are left with an eager streaming implementation that uses a queue.
async fn op_http_set_send_response_body_write_stream(
    state: Rc<RefCell<OpState>>,
    _: (),
    _: (),
) -> Result<u32, AnyError> {
    let events_rc = Rc::clone(state.borrow().borrow::<Rc<RefCell<Events>>>());
    let mut events = events_rc.borrow_mut();

    // Get request from event.
    let (mut response, responder, path) = match events.http.as_mut() {
        Some(event) => (
            mem::take(&mut event.response), // Take ownership of response.
            &mut event.responder,
            &event.path,
        ),
        None => return errors::missing_error_t(r#"unsupported event, "HttpEvent""#),
    };

    // Check read permission.
    let permissions = Rc::clone(state.borrow().borrow::<Rc<RefCell<Permissions>>>());
    permissions
        .borrow()
        .check(HttpEvent::SendResponse, HttpEventPath::from(path))?;

    // Create queue.
    let shared_queue = Arc::new(Mutex::new(BufferQueue::new()));

    // Create a body writer as well.
    let writer = BodyWriteStream(Arc::clone(&shared_queue));

    // Add queue to resource table.
    let rid = state
        .borrow_mut()
        .resource_table
        .add(BufferQueueResource(shared_queue));

    // Create a stream body.
    *response.body_mut() = Body::wrap_stream(writer); //= Body::from(buf.to_vec());

    // Send response.
    responder.send_response(response).await?;

    Ok(rid)
}

async fn op_http_write_response_body_chunk(
    state: Rc<RefCell<OpState>>,
    rid: ResourceId,
    buf: ZeroCopyBuf,
) -> Result<(), AnyError> {
    let events_rc = Rc::clone(state.borrow().borrow::<Rc<RefCell<Events>>>());
    let events = events_rc.borrow_mut();

    // Get request from event.
    let path = match events.http.as_ref() {
        Some(event) => &event.path,
        None => return errors::missing_error_t(r#"unsupported event, "HttpEvent""#),
    };

    // Check read permission.
    let permissions = Rc::clone(state.borrow().borrow::<Rc<RefCell<Permissions>>>());
    permissions
        .borrow()
        .check(HttpEvent::ReadRequest, HttpEventPath::from(path))?;

    // Get buffer queue.
    let resource = state
        .borrow()
        .resource_table
        .get::<BufferQueueResource>(rid)?;

    // Making sure queue size limit is not exceeded.
    let queue_len = resource.0.lock().queue.len();
    if queue_len >= MAX_BUFFER_QUEUE_SIZE {
        return errors::limit_exceeded_error_t(
            "maximum response body streaming queue limit reached",
        );
    }

    // Push value into buffer queue.
    resource.0.lock().queue.push_back(buf.to_vec());

    // Wake the associated stream future.
    if let Some(waker) = &resource.0.lock().waker {
        waker.wake_by_ref();
    };

    Ok(())
}
