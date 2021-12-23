// Copyright 2021 the Gigamono authors. All rights reserved. Apache 2.0 license.

use crate::permissions::{PermissionType, PermissionTypeKey};
use std::any::TypeId;

#[derive(Debug, Clone)]
pub enum HttpEvent {
    ReadRequest,
    ModifyRequest,
    WriteResponse,
    SendResponse,
}

impl PermissionType for HttpEvent {
    fn get_key<'a>(&self) -> PermissionTypeKey {
        PermissionTypeKey {
            type_id: TypeId::of::<Self>(),
            variant: 0,
        }
    }
}

impl Into<Box<dyn PermissionType>> for HttpEvent {
    fn into(self) -> Box<dyn PermissionType> {
        Box::new(self)
    }
}
