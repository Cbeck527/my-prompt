pub mod error;
pub mod module_trait;
pub mod modules;
pub mod prompt;
pub mod style;

// Re-export main types and functions
pub use error::{PromptError, Result};
pub use module_trait::{Module, ModuleContext};
pub use style::AnsiStyle;

// Export for integration tests
pub use prompt::{PROMPT_FORMAT, PromptModule, TRANSIENT_FORMAT, render_prompt};
