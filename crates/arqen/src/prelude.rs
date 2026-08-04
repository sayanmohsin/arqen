//! Ergonomic re-exports for common Arqen types.
//!
//! Import everything with:
//!
//! ```rust,ignore
//! use arqen::prelude::*;
//! ```

pub use crate::agent::{JobMetadata, ToolEffect, ToolMetadata, ToolRegistry};
pub use crate::app::ArqenApp;
pub use crate::config::AppConfig;
pub use crate::core::{AppError, ErrorKind};
pub use crate::jobs::JobHandler;
pub use crate::module::{Module, ModuleContext, ModuleError, ModuleHealth};

pub use async_trait::async_trait;
