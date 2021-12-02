// Copyright 2021 the Gigamono authors. All rights reserved. Apache 2.0 license.

use super::HttpEvent;

#[derive(Default)]
pub struct Events {
    pub http: Option<HttpEvent>,
}
