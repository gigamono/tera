// Copyright 2021 the Gigamono authors. All rights reserved. Apache 2.0 license.

use std::pin::Pin;

use super::HttpEvent;
use futures_util::Future;
use utilities::{
    http::{Body, Response},
    result::Result,
};

#[derive(Default)]
pub struct Events {
    pub http: Option<HttpEvent>,
}

pub trait EventResponder {
    fn send_response(&self, response: Response<Body>) -> Pin<Box<dyn Future<Output = Result<()>>>>;
}
