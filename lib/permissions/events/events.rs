// use crate::permissions::{PermissionType, PermissionTypeKey, Resource};
// use deno_core::{error::Context, futures::FutureExt};
// use hashbrown::HashSet;
// use std::{
//     any::TypeId,
//     future::Future,
//     hash::{Hash, Hasher},
//     path::PathBuf,
//     pin::Pin,
// };
// use tokio::fs;
// use utilities::{errors, result::Result};

#[derive(Debug, Copy, Clone)]
pub enum Events {
    Recieve,
    Respond,
}

pub struct NoResource();
