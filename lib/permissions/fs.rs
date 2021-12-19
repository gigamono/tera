// Copyright 2021 the Gigamono authors. All rights reserved. Apache 2.0 license.

//! This permission type guards accessto the file system.
//! It does not support paths starting with "..".
//! https://fuchsia.googlesource.com/docs/+/d4f9b980f18fc6722b06abb693240b29abbbc9fc/dotdot.md

use super::{PermissionType, PermissionTypeKey, Resource, State};
use path_clean::PathClean;
use std::{any::TypeId, fs, path::PathBuf, rc::Rc};
use utilities::{
    errors,
    result::{Context, Result},
};

#[derive(Debug, Copy, Clone)]
pub enum Fs {
    Open,
    Create,
    Read,
    Write,
    Execute,
    Info,
}

#[derive(Clone, Debug, Hash, PartialEq, Eq)]
pub struct FsPath(pub String);

#[derive(Clone, Debug, Hash, PartialEq, Eq)]

pub struct FsRoot(pub String);

impl Fs {
    pub fn clean_path(filename: &str, root: &str) -> Result<PathBuf> {
        // ".." paths are not supported.
        if filename.starts_with("../") {
            return errors::new_error_t(format!(r#"no support for ".." paths, "{}""#, filename));
        }

        let full_path = format!("{}/{}", root, filename);

        Ok(PathBuf::from(full_path).clean())
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

                let clean_full_dir =
                    &Self::clean_path(&dir.downcast_ref::<FsPath>().unwrap().0, root)?;

                // Relative path are not supported.
                if !(clean_full_dir.starts_with("/") || clean_full_dir.starts_with("\\")) {
                    return errors::new_error_t(format!(
                        r#"does not support non-absolute dirs in allow_list "{:?}""#,
                        clean_full_dir
                    ));
                }

                // Canonicalize dir.
                let canon_dir = fs::canonicalize(clean_full_dir).context(format!(
                    r#"canonicalizing allowed directory "{:?}""#,
                    clean_full_dir
                ))?;

                Ok(FsPath::from(canon_dir.display().to_string()).into())
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

        // Path resolution is different for Fs::Create as filename does not exist yet so we can't simply canonicalize on the filename. It will return an error.
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

impl From<&str> for FsPath {
    fn from(s: &str) -> Self {
        Self(s.into())
    }
}

impl From<&String> for FsPath {
    fn from(s: &String) -> Self {
        Self(s.into())
    }
}

impl From<String> for FsPath {
    fn from(s: String) -> Self {
        Self(s)
    }
}

impl AsRef<String> for FsPath {
    fn as_ref(&self) -> &String {
        &self.0
    }
}

impl From<&str> for FsRoot {
    fn from(s: &str) -> Self {
        Self(s.into())
    }
}

impl From<&String> for FsRoot {
    fn from(s: &String) -> Self {
        Self(s.into())
    }
}

impl From<String> for FsRoot {
    fn from(s: String) -> Self {
        Self(s)
    }
}

impl AsRef<String> for FsRoot {
    fn as_ref(&self) -> &String {
        &self.0
    }
}
