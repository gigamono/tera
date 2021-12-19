// Copyright 2021 the Gigamono authors. All rights reserved. Apache 2.0 license.

use futures_util::FutureExt;
use std::{future::Future, pin::Pin, rc::Rc};
use tokio::sync::mpsc::Sender;
use utilities::errors;
use utilities::hyper::{Body, Request, Response};
use utilities::result::Result;

use super::EventResponder;

pub struct HttpEvent {
    pub request: Request<Body>,
    pub response: Response<Body>,
    pub path: String,

    pub responder: Rc<dyn EventResponder>,
}

impl HttpEvent {
    pub fn new(request: Request<Body>, responder: Rc<dyn EventResponder>) -> Self {
        let response = Response::default();
        let path = request.uri().path().to_owned();

        Self {
            request,
            response,
            path,
            responder,
        }
    }
}

pub struct HttpResponder {
    response_tx: Rc<Sender<Response<Body>>>,
}

impl HttpResponder {
    pub fn new(response_tx: Rc<Sender<Response<Body>>>) -> Self {
        Self { response_tx }
    }
}

impl EventResponder for HttpResponder {
    fn send_response(&self, response: Response<Body>) -> Pin<Box<dyn Future<Output = Result<()>>>> {
        let response_tx = Rc::clone(&self.response_tx);

        async move {
            if let Err(err) = response_tx.send(response).await {
                return errors::new_error_t(format!("{:?}", err));
            }

            Ok(())
        }
        .boxed_local()
    }
}
