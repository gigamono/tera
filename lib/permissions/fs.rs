// Copyright 2021 the Gigamono authors. All rights reserved. Apache 2.0 license.

use super::{PermissionType, PermissionTypeKey, Resource};
use deno_core::futures::FutureExt;
use path_clean::PathClean;
use std::{any::TypeId, future::Future, path::PathBuf, pin::Pin};
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
}

#[derive(Clone, Debug, Hash, PartialEq, Eq)]
pub struct PathString(pub String);

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

    fn check(
        &self,
        _: &PermissionTypeKey,
        filename: &Box<dyn Resource>,
        allow_list: &Vec<Box<dyn Resource>>,
    ) -> Pin<Box<dyn Future<Output = Result<()>>>> {
        // Downcast trait object to PathString.
        let filename = filename.downcast_ref::<PathString>().unwrap().0.clone();

        // Get clones of permission type and allow_list to move into async block.
        let permission_type = *self;
        let allow_list = allow_list.clone();

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
            for allowed_dir in allow_list.into_iter() {
                // Downcast trait object to PathString.
                let allowed_dir = allowed_dir.downcast::<PathString>().unwrap().0;

                // SEC: Must canonoicalize path before matching.
                let canon_dir = fs::canonicalize(&allowed_dir).await?;

                if path.starts_with(canon_dir) {
                    found = true;
                    break;
                }
            }

            // Error if no filename is not in any permitted directory.
            if !found {
                errors::permission_error(format!(
                    r#"permission type "{}" does not exist for file {:?}"#,
                    permission_type.get_type(),
                    filename
                ))?
            }

            Ok(())
        }
        .boxed_local()
    }
}

impl Resource for PathString {
    fn get_clone(&self) -> Box<dyn Resource> {
        Box::new(self.clone())
    }

    fn get_debug(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_tuple("PathString").field(&self.0).finish()
    }
}

impl Into<Box<dyn PermissionType>> for FS {
    fn into(self) -> Box<dyn PermissionType> {
        Box::new(self)
    }
}

impl Into<Box<dyn Resource>> for PathString {
    fn into(self) -> Box<dyn Resource> {
        Box::new(self)
    }
}
