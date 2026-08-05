use crate::module_trait::{Module, ModuleContext};
use crate::modules::{character, claude, direnv, envs, fail, git, path, time, username};
use rayon::prelude::*;

#[derive(Debug)]
pub(crate) enum PromptModule {
    Character,
    Claude,
    Direnv,
    Fail,
    Envs,
    Git,
    Path,
    Time,
    Username,
}

impl PromptModule {
    fn render(&self, context: &ModuleContext) -> Option<String> {
        match self {
            Self::Character => character::CharacterModule.render(context),
            Self::Claude => claude::ClaudeModule.render(context),
            Self::Direnv => direnv::DirenvModule.render(context),
            Self::Fail => fail::FailModule.render(context),
            Self::Envs => envs::EnvsModule.render(context),
            Self::Git => git::GitModule.render(context),
            Self::Path => path::PathModule.render(context),
            Self::Time => time::TimeModule.render(context),
            Self::Username => username::UsernameModule.render(context),
        }
    }
}

pub(crate) const PROMPT_FORMAT: &[PromptModule] = &[
    PromptModule::Fail,
    PromptModule::Username,
    PromptModule::Path,
    PromptModule::Envs,
    PromptModule::Direnv,
    PromptModule::Git,
    PromptModule::Character,
];

pub(crate) const TRANSIENT_FORMAT: &[PromptModule] = &[PromptModule::Time, PromptModule::Character];

pub(crate) const CLAUDE_FORMAT: &[PromptModule] = &[
    PromptModule::Path,
    PromptModule::Envs,
    PromptModule::Direnv,
    PromptModule::Git,
    PromptModule::Claude,
];

/// Maximum number of Rayon worker threads used by prompt rendering.
const MAX_RAYON_THREADS: usize = 4;

/// Renders the given modules in parallel and combines the available output in order.
pub(crate) fn render_prompt(modules: &[PromptModule], context: &ModuleContext) -> String {
    let parts: Vec<_> = modules
        .par_iter()
        .map(|module| module.render(context))
        .collect();

    let mut output = String::new();
    for text in parts.into_iter().flatten() {
        output.push_str(&text);
    }

    output
}

/// Initializes the global Rayon pool for prompt rendering.
///
/// # Errors
///
/// Returns the Rayon initialization error when another global pool was already
/// initialized or when Rayon cannot create the requested worker threads.
pub(crate) fn init_thread_pool() -> Result<usize, rayon::ThreadPoolBuildError> {
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
    fn render_prompt_returns_empty_string_without_modules() {
        assert_eq!(render_prompt(&[], &ModuleContext::default()), "");
    }

    #[test]
    fn render_prompt_returns_output_for_default_format() {
        let context = ModuleContext {
            exit_code: Some(0),
            no_color: true,
            ..ModuleContext::default()
        };

        assert!(!render_prompt(PROMPT_FORMAT, &context).is_empty());
    }

    #[test]
    fn render_prompt_returns_output_for_transient_format() {
        let context = ModuleContext {
            exit_code: Some(0),
            no_color: true,
            ..ModuleContext::default()
        };

        assert!(!render_prompt(TRANSIENT_FORMAT, &context).is_empty());
    }

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
