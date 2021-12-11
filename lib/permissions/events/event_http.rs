// Copyright 2021 the Gigamono authors. All rights reserved. Apache 2.0 license.

use crate::permissions::{PermissionType, PermissionTypeKey, Resource};
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
pub struct UrlPathString(pub String);

impl HttpEvent {
    // Accepts url path patterns like `/v1/users/*`, `/v1/*/1/*`, `/v1/users*`
    fn url_path_match(pattern: &str, test: &UrlPathString) -> Result<bool> {
        let regex_pattern = regex::escape(&format!("^{}$", pattern.replace("*", ".*")));
        let re =
            Regex::new(&regex_pattern).context("compiling regex pattern for matching url path")?;

        if re.is_match(&test.as_ref()) {
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

    fn check_sync(
        &self,
        path: &Box<dyn Resource>,
        allow_list: Rc<Vec<Box<dyn Resource>>>,
    ) -> Result<()> {
        // Downcast trait object to PathString.
        let path = path.downcast_ref::<UrlPathString>().unwrap();

        // Check if `path` matches any path in allow_list.
        let mut found = false;
        for allowed_pattern in allow_list.iter() {
            // Downcast trait object to UrlPathString.
            let allowed_pattern = &allowed_pattern.downcast_ref::<UrlPathString>().unwrap().0;

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

impl Resource for UrlPathString {
    fn get_clone(&self) -> Box<dyn Resource> {
        Box::new(self.clone())
    }

    fn get_debug(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_tuple("PathString").field(&self.0).finish()
    }
}

impl Into<Box<dyn PermissionType>> for HttpEvent {
    fn into(self) -> Box<dyn PermissionType> {
        Box::new(self)
    }
}

impl Into<Box<dyn Resource>> for UrlPathString {
    fn into(self) -> Box<dyn Resource> {
        Box::new(self)
    }
}

impl From<&str> for UrlPathString {
    fn from(s: &str) -> Self {
        Self(s.into())
    }
}

impl From<&String> for UrlPathString {
    fn from(s: &String) -> Self {
        Self(s.into())
    }
}

impl AsRef<String> for UrlPathString {
    fn as_ref(&self) -> &String {
        &self.0
    }
}
