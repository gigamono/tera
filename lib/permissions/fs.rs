use super::{PermissionType, PermissionTypeKey, PermissionsBuilder};
use deno_core::error::Context;
use hashbrown::HashSet;
use std::{any::TypeId, fs, path::PathBuf};
use utilities::{errors, result::Result};

pub trait FSCapability {
    /// `allow_list` is a list of directory that a module has been granted access to.
    /// The type of access still largely depends on the types of permissions.
    /// SEC: The content of `allow_list` are expected to be in their canonical form.
    fn fs(self, permission: FS, allow_list: HashSet<String>) -> Self;
}

#[derive(Debug, Copy, Clone)]
pub enum FS {
    Open,
    Create,
    Read,
    Write,
}

impl FS {
    fn resolve_fs_create_path(filename: &str) -> Result<PathBuf> {
        // Used blocking std::fs here.
        // Error message
        let err_msg = format!(r#"canonicalizing path for, {:?}"#, filename);

        let path = if filename.starts_with("/") {
            PathBuf::from(filename)
        } else if filename.starts_with("../") {
            fs::canonicalize("../").context(err_msg)?.join(filename)
        } else {
            fs::canonicalize("./").context(err_msg)?.join(filename)
        };

        Ok(path)
    }
}

impl PermissionType for FS {
    #[inline]
    fn get_key<'a>(&self) -> PermissionTypeKey {
        PermissionTypeKey {
            type_id: TypeId::of::<Self>(),
            variant: *self as i32,
        }
    }

    fn check(
        &self,
        _: &PermissionTypeKey,
        filename: &str,
        allow_list: &HashSet<String>,
    ) -> Result<()> {
        // Path resolution is different for FS::Create as filename does not exist yet so we can't simply canonicalize.
        // Used blocking std::fs here.
        let path = if matches!(self, FS::Create) {
            Self::resolve_fs_create_path(filename)?
        } else {
            fs::canonicalize(filename).context(format!(r#"canonicalizing path {:?}"#, filename))?
        };

        // Check if `path` is a child of any dir in the allow_list.
        let mut found = false;
        for canon_dir in allow_list.iter() {
            if path.starts_with(canon_dir) {
                found = true;
                break;
            }
        }

        // Error if no filename is not in any permitted directory.
        if !found {
            errors::permission_error(format!(
                r#"permission type "{}" does not exist for file {:?}"#,
                self.get_type(),
                filename
            ))?
        }

        Ok(())
    }
}

impl Into<Box<dyn PermissionType>> for FS {
    fn into(self) -> Box<dyn PermissionType> {
        Box::new(self)
    }
}

impl FSCapability for PermissionsBuilder {
    fn fs(mut self, permission: FS, allow_list: HashSet<String>) -> Self {
        let permission_key = permission.get_key();
        self.0.insert(permission_key, allow_list);

        self
    }
}
