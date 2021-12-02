// Copyright 2021 the Gigamono authors. All rights reserved. Apache 2.0 license.

use std::{pin::Pin, rc::Rc};

use utilities::{
    http::{Request, Response},
    result::Result,
};

pub struct HttpEvent {
    pub request: Request,
    pub streamer: Rc<dyn HttpEventStreamer>,
}

// TODO: Use hyper Request and Response now.
pub trait HttpEventStreamer {
    fn read_request_body(&self) -> Pin<Box<Result<Vec<u8>>>>;

    fn write_response_body(&self, response: Response) -> Pin<Box<Result<()>>>;
}
