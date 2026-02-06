use crate::error::Result;
use crate::module_trait::{Module, ModuleContext};
use crate::modules::{character, claude, direnv, envs, fail, git, hostname, path, time, username};
use rayon::prelude::*;

#[derive(Debug, Clone)]
pub enum PromptModule {
    Character,
    Claude,
    Direnv,
    Fail,
    Envs,
    Git,
    #[allow(dead_code)] // I might use hostname in the future...
    Hostname,
    Path,
    Time,
    Username,
}

impl PromptModule {
    fn render(&self, context: &ModuleContext) -> Result<Option<String>> {
        match self {
            Self::Character => character::CharacterModule::new().render(context),
            Self::Claude => claude::ClaudeModule::new().render(context),
            Self::Direnv => direnv::DirenvModule::new().render(context),
            Self::Fail => fail::FailModule::new().render(context),
            Self::Envs => envs::EnvsModule::new().render(context),
            Self::Git => git::GitModule.render(context),
            Self::Hostname => hostname::HostnameModule::new().render(context),
            Self::Path => path::PathModule::new().render(context),
            Self::Time => time::TimeModule.render(context),
            Self::Username => username::UsernameModule::new().render(context),
        }
    }
}

pub const PROMPT_FORMAT: &[PromptModule] = &[
    PromptModule::Fail,
    PromptModule::Username,
    PromptModule::Path,
    PromptModule::Envs,
    PromptModule::Direnv,
    PromptModule::Git,
    PromptModule::Character,
];

pub const TRANSIENT_FORMAT: &[PromptModule] = &[PromptModule::Time, PromptModule::Character];

pub const CLAUDE_FORMAT: &[PromptModule] = &[
    PromptModule::Fail,
    PromptModule::Path,
    PromptModule::Envs,
    PromptModule::Direnv,
    PromptModule::Git,
    PromptModule::Claude,
];

/// Renders the given modules in parallel and combines their output.
///
/// # Errors
///
/// Returns `PromptError::ExternalCommandFailed` if any module's external command fails.
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
