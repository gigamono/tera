use std::any::{type_name, TypeId};
use std::collections::BTreeMap;
use utilities::result::{Context, Result};

#[derive(PartialEq, Eq, PartialOrd, Ord)]
pub struct PermissionTypeKey {
    pub type_id: TypeId,
    pub variant: i32,
}

pub trait PermissionType: std::fmt::Debug {
    fn get_key<'a>(&self) -> PermissionTypeKey;

    fn get_type<'a>(&self) -> String {
        format!("{}::{:?}", type_name::<Self>(), self)
    }
}

type PermissionMap = BTreeMap<PermissionTypeKey, PermissionChecker>;

type PermissionCheckFn = dyn Fn(&Box<dyn PermissionType>, &PermissionTypeKey, &str, &Vec<String>) -> Result<()>;

pub struct PermissionChecker {
    pub permitted_canon_resources: Vec<String>,
    pub check_fn: Box<PermissionCheckFn>,
}

#[derive(Default)]
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
        let checker = self.0.get(permission_key).context(format!(
            r#"permission type "{}" does not exist for resource {:?}"#,
            permission.get_type(),
            resource
        ))?;

        // Call stored permission checking function.
        (checker.check_fn)(permission, permission_key, resource, &checker.permitted_canon_resources)?;

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

impl std::fmt::Debug for PermissionChecker {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("PermissionChecker")
            .field("permitted_resources", &self.permitted_canon_resources)
            .finish()
    }
}
