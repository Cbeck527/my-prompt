use crate::error::Result;

#[derive(Debug, Default, Clone)]
pub struct ModuleContext {
    pub exit_code: Option<i32>,
    pub no_color: bool,
}

pub trait Module: Send + Sync {
    fn render(&self, context: &ModuleContext) -> Result<Option<String>>;
}
