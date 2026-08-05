use std::path::{Path, PathBuf};

use serde::Deserialize;

use crate::module_trait::{Module, ModuleContext};
use crate::modules::utils;
pub(crate) struct DirenvModule;

impl DirenvModule {
    fn render_with_direnv_file(
        direnv_file: Option<&Path>,
        context: &ModuleContext,
    ) -> Option<String> {
        let direnv_file = direnv_file?;
        let state = get_direnv_state(direnv_file, context.direnv_status_json.as_deref())?;

        Some(render_direnv_state(state, context))
    }
}

fn render_direnv_state(state: DirenvState, context: &ModuleContext) -> String {
    let text = match state {
        DirenvState::Allowed => "+direnv",
        DirenvState::Blocked => "!direnv",
    };

    if context.no_color {
        format!("[{text}] ")
    } else {
        use crate::style::{AnsiStyle, Color};
        let cyan = AnsiStyle::new(Color::Cyan, false);

        match state {
            DirenvState::Allowed => {
                format!("{}[{}]{} ", cyan.start_codes(), text, AnsiStyle::RESET)
            }
            DirenvState::Blocked => {
                let bold_red = AnsiStyle::new(Color::Red, true);
                format!(
                    "{}[{}{}{}]{} ",
                    cyan.start_codes(),
                    bold_red.start_codes(),
                    text,
                    cyan.start_codes(),
                    AnsiStyle::RESET,
                )
            }
        }
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
enum DirenvState {
    Allowed,
    Blocked,
}

impl Module for DirenvModule {
    fn render(&self, context: &ModuleContext) -> Option<String> {
        let direnv_file = utils::find_upward(".envrc");
        Self::render_with_direnv_file(direnv_file.as_deref(), context)
    }
}

#[derive(Deserialize)]
struct DirenvStatusRc {
    path: PathBuf,
    allowed: u8,
}

#[derive(Deserialize)]
struct DirenvStatusState {
    #[serde(rename = "foundRC")]
    found_rc: Option<DirenvStatusRc>,
}

#[derive(Deserialize)]
struct DirenvStatus {
    state: DirenvStatusState,
}

fn parse_direnv_status(status_text: &str, direnv_file: &Path) -> Option<DirenvState> {
    let status = serde_json::from_str::<DirenvStatus>(status_text).ok()?;
    let found_rc = status.state.found_rc?;

    if found_rc.path.as_path() != direnv_file {
        return None;
    }

    match found_rc.allowed {
        0 => Some(DirenvState::Allowed),
        1 | 2 => Some(DirenvState::Blocked),
        _ => None,
    }
}

pub(crate) fn benchmark_label(
    direnv_file: Option<&Path>,
    cached_status_json: Option<&str>,
) -> &'static str {
    let Some(direnv_file) = direnv_file else {
        return "no .envrc";
    };

    if cached_status_json
        .and_then(|status_json| parse_direnv_status(status_json, direnv_file))
        .is_some()
    {
        "shell cache"
    } else {
        "external status"
    }
}

fn get_direnv_state(direnv_file: &Path, cached_status_json: Option<&str>) -> Option<DirenvState> {
    cached_status_json
        .and_then(|status_json| parse_direnv_status(status_json, direnv_file))
        .or_else(|| get_direnv_status_slow(direnv_file))
}

fn get_direnv_status_slow(direnv_file: &Path) -> Option<DirenvState> {
    let direnv_root = direnv_file.parent()?;
    let Ok(output) = std::process::Command::new("direnv")
        .arg("status")
        .arg("--json")
        .current_dir(direnv_root)
        .output()
    else {
        return None;
    };

    if !output.status.success() {
        return None;
    }

    let status_text = std::str::from_utf8(&output.stdout).ok()?;
    parse_direnv_status(status_text, direnv_file)
}

#[cfg(test)]
mod tests {
    use super::*;

    fn status_json(
        direnv_file: &Path,
        found_allowed: Option<u8>,
        loaded_allowed: Option<u8>,
    ) -> String {
        let rc = |allowed| {
            serde_json::json!({
                "path": direnv_file,
                "allowed": allowed,
            })
        };

        serde_json::json!({
            "state": {
                "foundRC": found_allowed.map(rc),
                "loadedRC": loaded_allowed.map(rc),
            }
        })
        .to_string()
    }

    #[test]
    fn parses_allowed_found_rc_when_loaded_rc_is_null() {
        let direnv_file = Path::new("/tmp/project/.envrc");
        let status = status_json(direnv_file, Some(0), None);

        assert_eq!(
            parse_direnv_status(&status, direnv_file),
            Some(DirenvState::Allowed)
        );
    }

    #[test]
    fn parses_not_allowed_found_rc_as_blocked() {
        let direnv_file = Path::new("/tmp/project/.envrc");
        let status = status_json(direnv_file, Some(1), None);

        assert_eq!(
            parse_direnv_status(&status, direnv_file),
            Some(DirenvState::Blocked)
        );
    }

    #[test]
    fn parses_denied_found_rc_as_blocked() {
        let direnv_file = Path::new("/tmp/project/.envrc");
        let status = status_json(direnv_file, Some(2), None);

        assert_eq!(
            parse_direnv_status(&status, direnv_file),
            Some(DirenvState::Blocked)
        );
    }

    #[test]
    fn returns_none_when_found_rc_is_null() {
        let direnv_file = Path::new("/tmp/project/.envrc");
        let status = status_json(direnv_file, None, None);

        assert_eq!(parse_direnv_status(&status, direnv_file), None);
    }

    #[test]
    fn returns_none_for_malformed_status_json() {
        let direnv_file = Path::new("/tmp/project/.envrc");

        assert_eq!(parse_direnv_status("not json", direnv_file), None);
    }

    #[test]
    fn returns_none_for_unknown_allowed_value() {
        let direnv_file = Path::new("/tmp/project/.envrc");
        let status = status_json(direnv_file, Some(3), None);

        assert_eq!(parse_direnv_status(&status, direnv_file), None);
    }

    #[test]
    fn returns_none_when_found_rc_path_does_not_match() {
        let direnv_file = Path::new("/tmp/project/.envrc");
        let other_direnv_file = Path::new("/tmp/other/.envrc");
        let status = status_json(other_direnv_file, Some(0), None);

        assert_eq!(parse_direnv_status(&status, direnv_file), None);
    }

    #[test]
    fn module_does_not_render_without_an_envrc() {
        let context = ModuleContext::default();
        let temp_dir = tempfile::tempdir().expect("create temporary directory");
        let direnv_file = temp_dir.path().join(".envrc");

        let result = DirenvModule::render_with_direnv_file(
            direnv_file.exists().then_some(direnv_file.as_path()),
            &context,
        );
        assert!(
            result.is_none(),
            "Direnv module shouldn't render without .envrc"
        );
    }

    #[test]
    fn benchmark_reports_no_envrc_when_no_file_was_found() {
        assert_eq!(benchmark_label(None, None), "no .envrc");
    }

    #[test]
    fn benchmark_reports_shell_cache_only_for_a_matching_valid_status() {
        let direnv_file = Path::new("/tmp/project/.envrc");
        let matching_status = status_json(direnv_file, Some(0), None);
        let other_status = status_json(Path::new("/tmp/other/.envrc"), Some(0), None);

        assert_eq!(
            benchmark_label(Some(direnv_file), Some(&matching_status)),
            "shell cache"
        );
        assert_eq!(
            benchmark_label(Some(direnv_file), Some(&other_status)),
            "external status"
        );
        assert_eq!(
            benchmark_label(Some(direnv_file), Some("not json")),
            "external status"
        );
    }

    #[test]
    fn benchmark_reports_external_status_without_a_usable_cache() {
        let direnv_file = Path::new("/tmp/project/.envrc");

        assert_eq!(benchmark_label(Some(direnv_file), None), "external status");
    }
}
