use deno_core::ModuleLoader;

pub struct ESMLoader();

pub fn esm() -> ESMLoader {
    ESMLoader::new()
}

impl ESMLoader {
    pub fn new() -> Self {
        Self()
    }
}

impl ModuleLoader for ESMLoader {
    fn resolve(
        &self,
        specifier: &str,
        referrer: &str,
        _is_main: bool,
    ) -> Result<deno_core::ModuleSpecifier, deno_core::error::AnyError> {
        todo!()
    }

    fn load(
        &self,
        module_specifier: &deno_core::ModuleSpecifier,
        maybe_referrer: Option<deno_core::ModuleSpecifier>,
        is_dyn_import: bool,
    ) -> std::pin::Pin<Box<deno_core::ModuleSourceFuture>> {
        todo!()
    }
}
