// Copyright 2021 the Gigamono authors. All rights reserved. Apache 2.0 license.

use crate::permissions::{PermissionType, PermissionTypeKey, Resource, State};
use regex::Regex;
use std::{any::TypeId, rc::Rc};
use utilities::{
    errors,
    result::{Context, Result},
};

#[derive(Debug, Copy, Clone)]
pub enum HttpEvent {
    ReadRequest,
    ModifyRequest,
    WriteResponse,
    SendResponse,
}

#[derive(Clone, Debug, Hash, PartialEq, Eq)]
pub struct HttpEventPath(pub String);

impl HttpEvent {
    /// Accepts url path patterns like `/users/*`, `/users/*/name`, `/v1/users*`
    fn url_path_match(pattern: &str, test: &HttpEventPath) -> Result<bool> {
        // * does not consume /. For example, /users/*/name won't match /users/1/some/secret/name but it will match /users/1/name.
        // ** consumes /, so /users/*/name will match /users/1/some/secret/name.
        let regex_pattern = &format!(
            "^{}$",
            pattern.replace("**", "[^?#]+").replace("*", "[^/?#]+")
        );
        let re =
            Regex::new(&regex_pattern).context("compiling regex pattern for matching url path")?;

        if re.is_match(test.as_ref()) {
            return Ok(true);
        }

        Ok(false)
    }
}

impl PermissionType for HttpEvent {
    fn get_key<'a>(&self) -> PermissionTypeKey {
        PermissionTypeKey {
            type_id: TypeId::of::<Self>(),
            variant: 0,
        }
    }

    fn check(
        &self,
        path: &Box<dyn Resource>,
        allow_list: Rc<Vec<Box<dyn Resource>>>,
        _: &Option<Box<dyn State>>,
    ) -> Result<()> {
        // Downcast trait object to PathString.
        let path = path.downcast_ref::<HttpEventPath>().unwrap();

        // Check if `path` matches any path in allow_list.
        let mut found = false;
        for allowed_pattern in allow_list.iter() {
            // Downcast trait object to Path.
            let allowed_pattern = &allowed_pattern.downcast_ref::<HttpEventPath>().unwrap().0;

            if Self::url_path_match(allowed_pattern, &path)? {
                found = true;
                break;
            }
        }

        if !found {
            return errors::permission_error_t(format!(
                r#"permission type "{}" does not exist for path {:?}"#,
                self.get_type(),
                path
            ));
        }

        Ok(())
    }
}

impl Resource for HttpEventPath {
    fn get_clone(&self) -> Box<dyn Resource> {
        Box::new(self.clone())
    }

    fn get_debug(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_tuple("HttpEvent").field(&self.0).finish()
    }
}

impl Into<Box<dyn PermissionType>> for HttpEvent {
    fn into(self) -> Box<dyn PermissionType> {
        Box::new(self)
    }
}

impl Into<Box<dyn Resource>> for HttpEventPath {
    fn into(self) -> Box<dyn Resource> {
        Box::new(self)
    }
}

impl From<&str> for HttpEventPath {
    fn from(s: &str) -> Self {
        Self(s.into())
    }
}

impl From<&String> for HttpEventPath {
    fn from(s: &String) -> Self {
        Self(s.into())
    }
}

impl AsRef<String> for HttpEventPath {
    fn as_ref(&self) -> &String {
        &self.0
    }
}
