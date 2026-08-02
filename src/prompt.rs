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
    PromptModule::Path,
    PromptModule::Envs,
    PromptModule::Direnv,
    PromptModule::Git,
    PromptModule::Claude,
];

/// Maximum number of Rayon worker threads used by prompt rendering.
pub const MAX_RAYON_THREADS: usize = 4;

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

/// Initializes the global Rayon pool for prompt rendering.
///
/// # Errors
///
/// Returns the Rayon initialization error when another global pool was already
/// initialized or when Rayon cannot create the requested worker threads.
pub fn init_thread_pool() -> std::result::Result<usize, rayon::ThreadPoolBuildError> {
    let available_threads =
        std::thread::available_parallelism().map_or(1, std::num::NonZeroUsize::get);
    let requested_threads = std::env::var("RAYON_NUM_THREADS")
        .ok()
        .and_then(|value| value.parse::<usize>().ok());
    let thread_count = configured_thread_count(available_threads, requested_threads);

    rayon::ThreadPoolBuilder::new()
        .num_threads(thread_count)
        .build_global()?;

    Ok(rayon::current_num_threads())
}

fn configured_thread_count(available_threads: usize, requested_threads: Option<usize>) -> usize {
    let available_threads = available_threads.max(1);
    let requested_threads = requested_threads
        .filter(|thread_count| *thread_count > 0)
        .unwrap_or(available_threads);

    requested_threads
        .min(available_threads)
        .clamp(1, MAX_RAYON_THREADS)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn configured_thread_count_uses_host_limit_by_default() {
        assert_eq!(configured_thread_count(2, None), 2);
        assert_eq!(configured_thread_count(16, None), MAX_RAYON_THREADS);
    }

    #[test]
    fn configured_thread_count_honors_request_within_host_limit() {
        assert_eq!(configured_thread_count(16, Some(1)), 1);
        assert_eq!(configured_thread_count(16, Some(3)), 3);
    }

    #[test]
    fn configured_thread_count_caps_request_and_rejects_zero() {
        assert_eq!(configured_thread_count(16, Some(16)), MAX_RAYON_THREADS);
        assert_eq!(configured_thread_count(2, Some(16)), 2);
        assert_eq!(configured_thread_count(4, Some(0)), 4);
    }
}
