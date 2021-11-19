// Copyright 2021 the Gigamono authors. All rights reserved. Apache 2.0 license.

use std::{fs, pin::Pin, rc::Rc};
use utilities::{errors, result::Context};

use deno_core::{futures::FutureExt, ModuleLoader, ModuleSource};

use crate::permissions::{
    fs::{PathString, FS},
    Permissions,
};

pub struct ESMLoader {
    permissions: Rc<Permissions>,
}

pub fn esm(permissions: Rc<Permissions>) -> ESMLoader {
    ESMLoader {
        permissions: permissions,
    }
}

impl ModuleLoader for ESMLoader {
    /// `referrer` is the base url of the module that imported the module now getting resolved.
    /// And since bare relative & absolute paths (e.g. "./xyz.js" or "/hello.js") are valid module specifiers, we need the base_url to resolve it into a valid URL with a scheme.
    /// See https://html.spec.whatwg.org/multipage/webappapis.html#resolve-a-module-specifier.
    fn resolve(
        &self,
        specifier: &str,
        referrer: &str,
        _is_main: bool,
    ) -> Result<deno_core::ModuleSpecifier, deno_core::error::AnyError> {
        Ok(deno_core::resolve_import(specifier, referrer)?)
    }

    // TODO(appcypher): SEC: Support caching. Modules can be hi-jacked at runtime.
    fn load(
        &self,
        module_specifier: &deno_core::ModuleSpecifier,
        _maybe_referrer: Option<deno_core::ModuleSpecifier>,
        _is_dyn_import: bool,
    ) -> Pin<Box<deno_core::ModuleSourceFuture>> {
        let module_specifier = module_specifier.clone();
        let permissions = self.permissions.clone();

        async move {
            // We only support "file" scheme for now.
            let module_scheme = module_specifier.scheme();
            if module_scheme != "file" {
                errors::any_error(format!(
                    r#"unsupported URL scheme in import "{}""#,
                    module_scheme
                ))?;
            }

            // Get file path from module specifier.
            let module_path = module_specifier
                .as_str()
                .strip_prefix("file://")
                .context("getting path from specifier")?;

            // Check permissions.
            permissions
                .check(FS::Read, PathString(module_path.into()))
                .await?;

            // Fetch module source.
            let code = fs::read_to_string(module_path)
                .context(format!(r#"reading module code from "{}""#, module_path))?;

            let mod_src = ModuleSource {
                code,
                module_url_specified: module_specifier.to_string(),
                module_url_found: module_specifier.to_string(),
            };

            Ok(mod_src)
        }
        .boxed_local()
    }
}
