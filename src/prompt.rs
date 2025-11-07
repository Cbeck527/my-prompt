use crate::error::Result;
use crate::module_trait::{Module, ModuleContext};
use crate::modules::{character, fail, git, hostname, path, time, username};
use rayon::prelude::*;

#[derive(Debug, Clone)]
pub enum PromptModule {
    Fail,
    Username,
    Path,
    Git,
    Time,
    Hostname,
    Character,
}

impl PromptModule {
    fn render(&self, context: &ModuleContext) -> Result<Option<String>> {
        match self {
            Self::Fail => fail::FailModule::new().render(context),
            Self::Username => username::UsernameModule::new().render(context),
            Self::Path => path::PathModule::new().render(context),
            Self::Git => git::GitModule.render(context),
            Self::Time => time::TimeModule.render(context),
            Self::Hostname => hostname::HostnameModule::new().render(context),
            Self::Character => character::CharacterModule::new().render(context),
        }
    }
}

pub const PROMPT_FORMAT: &[PromptModule] = &[
    PromptModule::Fail,
    PromptModule::Username,
    PromptModule::Path,
    PromptModule::Git,
    PromptModule::Character,
];

pub const TRANSIENT_FORMAT: &[PromptModule] = &[PromptModule::Time, PromptModule::Character];

pub fn render_prompt(modules: &[PromptModule], context: &ModuleContext) -> Result<String> {
    let parts: Vec<_> = modules.par_iter().map(|m| m.render(context)).collect();

    let mut output = String::new();
    for result in parts {
        if let Some(text) = result? {
            output.push_str(&text);
        }
    }

    Ok(output)
}

pub fn init_thread_pool() {
    let max_threads = std::cmp::min(rayon::current_num_threads(), 4);
    let _ = rayon::ThreadPoolBuilder::new()
        .num_threads(max_threads)
        .build_global();
}
