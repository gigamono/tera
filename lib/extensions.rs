mod fs;
mod event_http;

pub use deno_core::Extension; // Re-export
pub use fs::fs;
pub use event_http::event_http;
