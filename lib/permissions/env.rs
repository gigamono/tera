// Copyright 2021 the Gigamono authors. All rights reserved. GPL-3.0 License.

use crate::permissions::{PermissionType, PermissionTypeKey};
use std::any::TypeId;

#[derive(Debug, Clone)]
pub enum Env {
    Read,
}

impl PermissionType for Env {
    fn get_key<'a>(&self) -> PermissionTypeKey {
        PermissionTypeKey {
            type_id: TypeId::of::<Self>(),
            variant: 0,
        }
    }
}
