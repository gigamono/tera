use deno_core::ModuleLoader;

pub struct FileLoader();

pub fn dev() -> FileLoader {
    FileLoader::new()
}

impl FileLoader {
    pub fn new() -> Self {
        Self()
    }
}

impl ModuleLoader for FileLoader {
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
