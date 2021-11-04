use hashbrown::HashSet;
use std::any::{type_name, TypeId};
use std::collections::BTreeMap;
use utilities::errors;
use utilities::result::Result;

// TODO(appcypher):
//  Use HashSet<Box<dyn Resource>> so that we can pass values like PathBuf and resolve path to canonical in fs method.
//  impl Hash for Box<dyn Resource>
//  impl Resource for FS
//  pub trait Resource { fn downcast<T>(&self) }
type PermissionMap = BTreeMap<PermissionTypeKey, HashSet<String>>;

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
        resource: &str,
        allow_list: &HashSet<String>,
    ) -> Result<()>;
}

#[derive(Default, Debug)]
pub struct Permissions(PermissionMap);

pub struct PermissionsBuilder(pub PermissionMap);

impl Permissions {
    pub fn builder() -> PermissionsBuilder {
        PermissionsBuilder::new()
    }

    pub fn check(
        &self,
        permission: impl Into<Box<dyn PermissionType>>,
        resource: &str,
    ) -> Result<()> {
        let permission = &permission.into();
        let permission_key = &permission.get_key();

        // Check permission type exists.
        match self.0.get(permission_key) {
            None => errors::permission_error(format!(
                r#"permission type "{}" does not exist for file {:?}"#,
                permission.get_type(),
                resource
            ))?,
            Some(allow_list) => permission.check(permission_key, resource, &allow_list)?,
        }

        Ok(())
    }
}

impl PermissionsBuilder {
    pub fn new() -> Self {
        Self(BTreeMap::new())
    }

    pub fn build(self) -> Permissions {
        Permissions(self.0)
    }
}
