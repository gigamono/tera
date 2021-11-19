// Copyright 2021 the Gigamono authors. All rights reserved. Apache 2.0 license.

use downcast_rs::{impl_downcast, Downcast};
use std::any::{type_name, TypeId};
use std::cmp::Eq;
use std::collections::BTreeMap;
use std::fmt::Debug;
use std::future::Future;
use std::iter::FromIterator;
use std::pin::Pin;
use utilities::{errors, result::Result};

type PermissionMap = BTreeMap<PermissionTypeKey, Vec<Box<dyn Resource>>>;

pub trait Resource: Downcast {
    fn get_debug(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result;
    fn get_clone(&self) -> Box<dyn Resource>;
}

#[derive(PartialEq, Eq, PartialOrd, Ord, Debug)]
pub struct PermissionTypeKey {
    pub type_id: TypeId,
    pub variant: i32,
}

pub trait PermissionType: std::fmt::Debug {
    fn get_key<'a>(&self) -> PermissionTypeKey;

    fn get_type<'a>(&self) -> String {
        format!("{}::{:?}", type_name::<Self>(), self)
    }

    fn check(
        &self,
        permission_key: &PermissionTypeKey,
        resource: &Box<dyn Resource>,
        allow_list: &Vec<Box<dyn Resource>>,
    ) -> Pin<Box<dyn Future<Output = Result<()>>>>;
}

#[derive(Default, Debug)]
pub struct Permissions(PermissionMap);

pub struct PermissionsBuilder(pub(super) PermissionMap);

impl Permissions {
    pub fn builder() -> PermissionsBuilder {
        PermissionsBuilder::new()
    }

    pub async fn check(
        &self,
        permission: impl Into<Box<dyn PermissionType>>,
        resource: impl Into<Box<dyn Resource>>,
    ) -> Result<()> {
        let permission = &permission.into();
        let permission_key = &permission.get_key();
        let resource = &resource.into();

        // Check permission type exists.
        match self.0.get(permission_key) {
            None => errors::permission_error(format!(
                r#"permission type "{}" does not exist for file {:?}"#,
                permission.get_type(),
                resource
            ))?,
            Some(allow_list) => {
                permission
                    .check(permission_key, &resource, &allow_list)
                    .await?
            }
        }

        Ok(())
    }
}

impl PermissionsBuilder {
    pub fn new() -> Self {
        Self(BTreeMap::new())
    }

    pub fn add_permissions(
        mut self,
        permissions: &[(
            impl Into<Box<dyn PermissionType>> + Clone,
            &[impl Into<Box<dyn Resource>> + Clone],
        )],
    ) -> PermissionsBuilder {
        for (permission_type, resources) in permissions.iter() {
            // Construct resource hashset from allow_list.
            let allow_list = Vec::from_iter(resources.iter().map(|s| s.clone().into()));

            // Get permission key from permission type.
            let permission_type: Box<dyn PermissionType> = permission_type.clone().into();
            let permission_key = permission_type.get_key();

            // Add permission type.
            self.0.insert(permission_key, allow_list);
        }

        self
    }

    pub fn build(self) -> Permissions {
        Permissions(self.0)
    }
}

// === Impls ===

impl_downcast!(Resource);

impl std::fmt::Debug for Box<dyn Resource> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        self.get_debug(f)
    }
}

impl Clone for Box<dyn Resource> {
    fn clone(&self) -> Self {
        self.get_clone()
    }
}
