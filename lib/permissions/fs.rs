// Copyright 2021 the Gigamono authors. All rights reserved. Apache 2.0 license.

use super::{PermissionType, PermissionTypeKey, Resource, State};
use path_clean::PathClean;
use std::{
    any::TypeId,
    convert::TryFrom,
    fs,
    path::{Path, PathBuf},
    rc::Rc,
};
use utilities::{
    errors::{self, SystemError},
    result::{Context, Result},
};

/// The access levels of the Fs permission.
#[derive(Debug, Copy, Clone)]
pub enum Fs {
    Open,
    Create,
    Read,
    Write,
    Execute,
    Info,
}

/// Represents the resource path.
/// This is always relative to `FsRoot`.
///
/// Expects a relative `path` and does not support paths starting with `../`.
#[derive(Clone, Debug, Hash, PartialEq, Eq)]
pub struct FsPath(PathBuf);

/// Fs permission requires a root to be specified.
///
/// Path will be resolved to a canonical absolute path.
#[derive(Clone, Debug, Hash, PartialEq, Eq)]
pub struct FsRoot(PathBuf);

impl Fs {
    /// Joins specified path with its root to form an absolute path, then subsequently cleans it.
    ///
    /// Resolves `..`, `.` and removing excess separator in the absolute path.
    ///
    /// Expects a relative `path` and does not support paths starting with `../`.
    pub fn clean_path(path: &Path, root: &Path) -> Result<PathBuf> {
        // SEC: Paths starting with "../"  are not supported.
        // https://fuchsia.googlesource.com/docs/+/d4f9b980f18fc6722b06abb693240b29abbbc9fc/dotdot.md
        if path.starts_with(PathBuf::from("..")) {
            return errors::new_error_t(format!(r#"no support for ".." paths, {:?}"#, path));
        }

        // Join paths.
        let full_path: PathBuf = [root, path].iter().collect();

        Ok(full_path.clean())
    }
}

impl FsRoot {
    fn canonicalize(path: PathBuf) -> Result<PathBuf> {
        fs::canonicalize(&path).context(format!(
            r#"canonicalizing root dir of fs permission {:?}"#,
            path
        ))
    }
}

impl PermissionType for Fs {
    fn get_key<'a>(&self) -> PermissionTypeKey {
        PermissionTypeKey {
            type_id: TypeId::of::<Self>(),
            variant: *self as i32,
        }
    }

    fn map(
        &self,
        allow_list: Vec<Box<dyn Resource>>,
        state: &Option<Box<dyn State>>,
    ) -> Result<Vec<Box<dyn Resource>>> {
        // Canonicalize every dir in the allow list.
        let canon_list = allow_list
            .iter()
            .map(|dir| {
                // Downcast state to Root. Expects a root to be specified.
                let root = if let Some(state) = &state {
                    state.downcast_ref::<FsRoot>().unwrap().as_ref()
                } else {
                    return errors::permission_error_t("root path not specified");
                };

                let path = dir.downcast_ref::<FsPath>().unwrap().as_ref();

                let clean_full_dir = &Self::clean_path(path, root)?;

                // Canonicalize dir.
                let canon_dir = fs::canonicalize(clean_full_dir).context(format!(
                    r#"canonicalizing allowed directory {:?}"#,
                    clean_full_dir
                ))?;

                Ok(FsPath::from(canon_dir).into())
            })
            .collect::<Result<Vec<Box<dyn Resource>>>>()?;

        Ok(canon_list)
    }

    fn check(
        &self,
        filename: &Box<dyn Resource>,
        allow_list: Rc<Vec<Box<dyn Resource>>>,
        state: &Option<Box<dyn State>>,
    ) -> Result<()> {
        // Downcast state to Root. Expects a root to be specified.
        let root = if let Some(state) = state {
            state.downcast_ref::<FsRoot>().unwrap().as_ref()
        } else {
            return errors::permission_error_t("root path not specified");
        };

        // Downcast filename to Path.
        let filename = filename.downcast_ref::<FsPath>().unwrap().as_ref();

        // Clean path.
        let path = Self::clean_path(&filename, &root)?;

        // Check if `path` is a child of any dir in the allow_list.
        let mut found = false;
        for allowed_dir in allow_list.iter() {
            // Downcast trait object to Path.
            let canon_allowed_dir = &allowed_dir.downcast_ref::<FsPath>().unwrap().0;

            if path.starts_with(canon_allowed_dir) {
                found = true;
                break;
            }
        }

        if !found {
            return errors::permission_error_t(format!(
                r#"permission type "{}" does not exist for file {:?}"#,
                self.get_type(),
                filename
            ));
        }

        Ok(())
    }
}

impl Resource for FsPath {
    fn get_clone(&self) -> Box<dyn Resource> {
        Box::new(self.clone())
    }

    fn get_debug(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_tuple("FsPath").field(&self.0).finish()
    }
}

impl State for FsRoot {
    fn get_debug(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_tuple("FsRoot").field(&self.0).finish()
    }
}

impl Into<Box<dyn PermissionType>> for Fs {
    fn into(self) -> Box<dyn PermissionType> {
        Box::new(self)
    }
}

impl Into<Box<dyn Resource>> for FsPath {
    fn into(self) -> Box<dyn Resource> {
        Box::new(self)
    }
}

impl Into<Box<dyn State>> for FsRoot {
    fn into(self) -> Box<dyn State> {
        Box::new(self)
    }
}

impl From<&Path> for FsPath {
    fn from(path: &Path) -> Self {
        Self(path.into())
    }
}

impl From<&PathBuf> for FsPath {
    fn from(path: &PathBuf) -> Self {
        Self(path.into())
    }
}

impl From<PathBuf> for FsPath {
    fn from(path: PathBuf) -> Self {
        Self(path)
    }
}

impl From<&str> for FsPath {
    fn from(path: &str) -> Self {
        Self(path.into())
    }
}

impl From<&String> for FsPath {
    fn from(path: &String) -> Self {
        Self(path.into())
    }
}

impl From<String> for FsPath {
    fn from(path: String) -> Self {
        Self(path.into())
    }
}

impl AsRef<Path> for FsPath {
    fn as_ref(&self) -> &Path {
        &self.0
    }
}

impl TryFrom<&Path> for FsRoot {
    type Error = SystemError;

    fn try_from(path: &Path) -> Result<Self, Self::Error> {
        let value = Self::canonicalize(path.into())?;
        Ok(Self(value))
    }
}

impl TryFrom<&PathBuf> for FsRoot {
    type Error = SystemError;

    fn try_from(path: &PathBuf) -> Result<Self, Self::Error> {
        let value = Self::canonicalize(path.into())?;
        Ok(Self(value))
    }
}

impl TryFrom<PathBuf> for FsRoot {
    type Error = SystemError;

    fn try_from(path: PathBuf) -> Result<Self, Self::Error> {
        let value = Self::canonicalize(path)?;
        Ok(Self(value))
    }
}

impl TryFrom<&str> for FsRoot {
    type Error = SystemError;

    fn try_from(path: &str) -> Result<Self, Self::Error> {
        let value = Self::canonicalize(path.into())?;
        Ok(Self(value))
    }
}

impl TryFrom<&String> for FsRoot {
    type Error = SystemError;

    fn try_from(path: &String) -> Result<Self, Self::Error> {
        let value = Self::canonicalize(path.into())?;
        Ok(Self(value))
    }
}

impl TryFrom<String> for FsRoot {
    type Error = SystemError;

    fn try_from(path: String) -> Result<Self, Self::Error> {
        let value = Self::canonicalize(path.into())?;
        Ok(Self(value))
    }
}

impl AsRef<Path> for FsRoot {
    fn as_ref(&self) -> &Path {
        &self.0
    }
}
