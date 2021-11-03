use super::{PermissionChecker, PermissionType, PermissionTypeKey, PermissionsBuilder};
use std::{any::TypeId, fs};
use utilities::{errors, result::Result};

pub trait FSCapability {
    fn fs(self, permission: FS, canon_dirs: Vec<String>) -> Self;
    fn permission_check(
        permission: &Box<dyn PermissionType>,
        permission_key: &PermissionTypeKey,
        filename: &str,
        permitted_canon_filenames: &Vec<String>,
    ) -> Result<()>;
}

#[derive(Debug, Copy, Clone)]
pub enum FS {
    Read,
    Write,
}

impl PermissionType for FS {
    #[inline]
    fn get_key<'a>(&self) -> PermissionTypeKey {
        PermissionTypeKey {
            type_id: TypeId::of::<Self>(),
            variant: *self as i32,
        }
    }
}

impl Into<Box<dyn PermissionType>> for FS {
    fn into(self) -> Box<dyn PermissionType> {
        Box::new(self)
    }
}

impl FSCapability for PermissionsBuilder {
    fn fs(mut self, permission: FS, canon_dirs: Vec<String>) -> Self {
        let permission_key = permission.get_key();
        self.0.insert(
            permission_key,
            PermissionChecker {
                permitted_canon_resources: canon_dirs,
                check_fn: Box::new(<Self as FSCapability>::permission_check),
            },
        );

        self
    }

    // TODO(appcypher): Use task::spawn_blocking for io calls? Will that cause it to be slower?
    fn permission_check(
        permission: &Box<dyn PermissionType>,
        _: &PermissionTypeKey,
        filename: &str,
        permitted_canon_dirs: &Vec<String>,
    ) -> Result<()> {
        // Canonicalize filename path.
        let canon_path = fs::canonicalize(filename)?;

        // Find a premitted directory that `canon_path` is a child of.
        let mut found = false;
        for canon_dir in permitted_canon_dirs.iter() {
            if canon_path.starts_with(canon_dir) {
                found = true;
                break;
            }
        }

        // Error if no filename is not in any permitted directory.
        if !found {
            errors::any_error(format!(
                r#"permission type "{}" does not exist for file {:?}"#,
                permission.get_type(),
                filename
            ))?
        }

        Ok(())
    }
}
