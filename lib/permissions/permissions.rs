// Copyright 2021 the Gigamono authors. All rights reserved. GPL-3.0 License.

use downcast_rs::{impl_downcast, Downcast};
use std::any::{type_name, TypeId};
use std::cmp::Eq;
use std::collections::BTreeMap;
use std::fmt::Debug;
use std::iter::FromIterator;
use std::rc::Rc;
use utilities::{errors, result::Result};

type PermissionMap = BTreeMap<PermissionTypeKey, Rc<Vec<Box<dyn Resource>>>>;

pub trait Resource: Downcast {
    fn get_debug(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result;
    fn get_clone(&self) -> Box<dyn Resource>;
}

pub trait State: Downcast {
    fn get_debug(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result;
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

    fn map(
        &self,
        allow_list: Vec<Box<dyn Resource>>,
        _state: &Option<Box<dyn State>>,
    ) -> Result<Vec<Box<dyn Resource>>> {
        Ok(allow_list)
    }

    fn check(
        &self,
        _resource: &Box<dyn Resource>,
        _allow_list: Rc<Vec<Box<dyn Resource>>>,
        _state: &Option<Box<dyn State>>,
    ) -> Result<()> {
        unimplemented!()
    }
}

#[derive(Default, Debug)]
pub struct Permissions {
    pub map: PermissionMap,
    pub state: Option<Box<dyn State>>,
}

pub struct PermissionsBuilder {
    pub(super) map: PermissionMap,
    pub(super) state: Option<Box<dyn State>>,
}

impl Permissions {
    pub fn builder() -> PermissionsBuilder {
        PermissionsBuilder::new()
    }

    pub fn check(
        &self,
        permission: impl Into<Box<dyn PermissionType>>,
        resource: impl Into<Box<dyn Resource>>,
    ) -> Result<()> {
        let permission = &permission.into();
        let permission_key = &permission.get_key();
        let resource = &resource.into();

        // Check permission type exists.
        match self.map.get(permission_key) {
            None => errors::permission_error_t(format!(
                r#"permission type "{}" does not exist for file {:?}"#,
                permission.get_type(),
                resource
            )),
            Some(allow_list) => permission.check(&resource, Rc::clone(allow_list), &self.state),
        }
    }

    pub fn check_exists(&self, permission: impl Into<Box<dyn PermissionType>>) -> Result<()> {
        let permission = &permission.into();
        let permission_key = &permission.get_key();

        // Check permission type exists.
        if let None = self.map.get(permission_key) {
            return errors::permission_error_t(format!(
                r#"permission type "{}" does not exist"#,
                permission.get_type(),
            ));
        }

        Ok(())
    }
}

impl PermissionsBuilder {
    pub fn new() -> Self {
        Self {
            map: BTreeMap::new(),
            state: None,
        }
    }

    pub fn add_state(mut self, state: impl Into<Box<dyn State>>) -> Self {
        self.state = Some(state.into());
        self
    }

    pub fn add_permissions_with_allow_lists(
        mut self,
        permissions: &[(
            impl Into<Box<dyn PermissionType>> + Clone,
            &[impl Into<Box<dyn Resource>> + Clone],
        )],
    ) -> Result<Self> {
        for (permission_type, resources) in permissions.iter() {
            // Construct resource hashset from allow_list.
            let allow_list = Vec::from_iter(resources.iter().map(|s| s.clone().into()));

            // Get permission key from permission type.
            let permission_type: Box<dyn PermissionType> = permission_type.clone().into();
            let permission_key = permission_type.get_key();

            // Do possibly custom stuff on allow list before saving.
            let allow_list = permission_type.map(allow_list, &self.state)?;

            // Add permission type.
            self.map.insert(permission_key, Rc::new(allow_list));
        }

        Ok(self)
    }

    pub fn add_owned_permissions_with_allow_lists(
        mut self,
        permissions: Vec<(Box<dyn PermissionType>, Vec<Box<dyn Resource>>)>,
    ) -> Result<Self> {
        for (permission_type, resources) in permissions.into_iter() {
            // Construct resource hashset from allow_list.
            let allow_list = Vec::from_iter(resources.into_iter().map(|s| s));

            // Get permission key from permission type.
            let permission_type: Box<dyn PermissionType> = permission_type;
            let permission_key = permission_type.get_key();

            // Do possibly custom stuff on allow list before saving.
            let allow_list = permission_type.map(allow_list, &self.state)?;

            // Add permission type.
            self.map.insert(permission_key, Rc::new(allow_list));
        }

        Ok(self)
    }

    pub fn add_permissions(
        mut self,
        permissions: &[impl Into<Box<dyn PermissionType>> + Clone],
    ) -> Result<Self> {
        for permission_type in permissions.iter() {
            // Get permission key from permission type.
            let permission_type: Box<dyn PermissionType> = permission_type.clone().into();
            let permission_key = permission_type.get_key();

            // Add permission type.
            self.map.insert(permission_key, Rc::new(vec![]));
        }

        Ok(self)
    }

    pub fn add_owned_permissions(
        mut self,
        permissions: Vec<Box<dyn PermissionType>>,
    ) -> Result<Self> {
        for permission_type in permissions.into_iter() {
            // Get permission key from permission type.
            let permission_type: Box<dyn PermissionType> = permission_type;
            let permission_key = permission_type.get_key();

            // Add permission type.
            self.map.insert(permission_key, Rc::new(vec![]));
        }

        Ok(self)
    }

    pub fn build(self) -> Permissions {
        Permissions {
            map: self.map,
            state: self.state,
        }
    }
}

// === Impls ===

impl_downcast!(Resource);

impl_downcast!(State);

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

impl std::fmt::Debug for Box<dyn State> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        self.get_debug(f)
    }
}
