// Copyright 2021 the Gigamono authors. All rights reserved. Apache 2.0 license.

use futures_util::FutureExt;
use std::{future::Future, pin::Pin, rc::Rc};
use tokio::sync::mpsc::Sender;
use utilities::errors;
use utilities::hyper::{Body, Request, Response};
use utilities::result::Result;

use super::EventResponder;

pub struct HttpEvent {
    pub request: Request<Body>,            // The working request.
    pub response: Response<Body>,          // The working response.
    pub responder: Rc<dyn EventResponder>, // The response sender implementation
}

impl HttpEvent {
    pub fn new(request: Request<Body>, responder: Rc<dyn EventResponder>) -> Self {
        Self {
            request,
            response: Response::default(),
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
