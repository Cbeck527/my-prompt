use crate::error::Result;

#[derive(Debug, Clone)]
pub struct ModuleContext {
    pub exit_code: Option<i32>,
    pub no_color: bool,
}

impl Default for ModuleContext {
    fn default() -> Self {
        Self {
            exit_code: None,
            no_color: false,
        }
    }
}

pub trait Module: Send + Sync {
    fn render(&self, context: &ModuleContext) -> Result<Option<String>>;
}
