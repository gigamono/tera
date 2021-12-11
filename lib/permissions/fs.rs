// Copyright 2021 the Gigamono authors. All rights reserved. Apache 2.0 license.

use super::{PermissionType, PermissionTypeKey, Resource};
use deno_core::futures::FutureExt;
use path_clean::PathClean;
use std::{any::TypeId, future::Future, path::PathBuf, pin::Pin, rc::Rc};
use tokio::fs;
use utilities::{
    errors,
    result::{Context, Result},
};

#[derive(Debug, Copy, Clone)]
pub enum FS {
    Open,
    Create,
    Read,
    Write,
    Import,
}

#[derive(Clone, Debug, Hash, PartialEq, Eq)]
pub struct FilePathString(pub String);

impl FS {
    async fn resolve_fs_create_path(filename: &str) -> Result<PathBuf> {
        // Error message
        let err_msg = format!(r#"canonicalizing path for, {:?}"#, filename);

        let path = if filename.starts_with("/") {
            // Clean path.
            PathBuf::from(filename).clean()
        } else if filename.starts_with("../") {
            // Canonicalize the part that we are sure exists.
            let prefix = fs::canonicalize("../").await.context(err_msg)?;

            // Get the remaining suffix.
            let suffix = filename.strip_prefix("../").unwrap();

            // Join and clean.
            prefix.join(suffix).clean()
        } else {
            // Canonicalize the part that we are sure exists.
            let prefix = fs::canonicalize("./").await.context(err_msg)?;

            // Join and clean.
            prefix.join(filename).clean()
        };

        Ok(path)
    }
}

impl PermissionType for FS {
    fn get_key<'a>(&self) -> PermissionTypeKey {
        PermissionTypeKey {
            type_id: TypeId::of::<Self>(),
            variant: *self as i32,
        }
    }

    // TODO(appcypher): SEC: Make filename and allow_list absolutes
    fn check(
        &self,
        filename: &Box<dyn Resource>,
        allow_list: Rc<Vec<Box<dyn Resource>>>,
    ) -> Pin<Box<dyn Future<Output = Result<()>>>> {
        // Downcast trait object to FilePathString.
        let filename = filename.downcast_ref::<FilePathString>().unwrap().as_ref().clone();

        // Get clone of permission type.
        let permission_type = *self;

        async move {
            // Path resolution is different for FS::Create as filename does not exist yet so we can't simply canonicalize on the filename. It will return an error.
            let path = if matches!(permission_type, FS::Create) {
                Self::resolve_fs_create_path(&filename).await?
            } else {
                fs::canonicalize(&filename)
                    .await
                    .context(format!(r#"canonicalizing path {:?}"#, filename))?
            };

            // Check if `path` is a child of any dir in the allow_list.
            let mut found = false;
            for allowed_dir in allow_list.iter() {
                // Downcast trait object to FilePathString.
                let allowed_dir = &allowed_dir.downcast_ref::<FilePathString>().unwrap().0;

                // SEC: Must canonoicalize path before matching.
                let canon_dir = fs::canonicalize(allowed_dir).await?;

                if path.starts_with(canon_dir) {
                    found = true;
                    break;
                }
            }

            if !found {
                return errors::permission_error_t(format!(
                    r#"permission type "{}" does not exist for file {:?}"#,
                    permission_type.get_type(),
                    filename
                ));
            }

            Ok(())
        }
        .boxed_local()
    }
}

impl Resource for FilePathString {
    fn get_clone(&self) -> Box<dyn Resource> {
        Box::new(self.clone())
    }

    fn get_debug(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_tuple("FilePathString").field(&self.0).finish()
    }
}

impl Into<Box<dyn PermissionType>> for FS {
    fn into(self) -> Box<dyn PermissionType> {
        Box::new(self)
    }
}

impl Into<Box<dyn Resource>> for FilePathString {
    fn into(self) -> Box<dyn Resource> {
        Box::new(self)
    }
}

impl From<&str> for FilePathString {
    fn from(s: &str) -> Self {
        Self(s.into())
    }
}

impl From<&String> for FilePathString {
    fn from(s: &String) -> Self {
        Self(s.into())
    }
}

impl AsRef<String> for FilePathString {
    fn as_ref(&self) -> &String {
        &self.0
    }
}
