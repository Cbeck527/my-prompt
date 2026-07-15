use serde::Deserialize;
use std::path::Path;

use crate::error::Result;
use crate::module_trait::{Module, ModuleContext};
use crate::modules::utils;
pub struct DirenvModule;

impl Default for DirenvModule {
    fn default() -> Self {
        Self::new()
    }
}

impl DirenvModule {
    #[must_use]
    pub fn new() -> Self {
        Self
    }

    fn render_with_direnv_file(
        direnv_file: Option<&Path>,
        context: &ModuleContext,
    ) -> Option<String> {
        let direnv_file = direnv_file?;
        let direnv_root = direnv_file.parent()?;
        let state = get_direnv_status_slow(direnv_root)?;

        let text = match state {
            DirenvState::Allowed => "+direnv",
            DirenvState::Blocked => "!direnv",
        };

        if context.no_color {
            Some(format!("[{text}] "))
        } else {
            use crate::style::{AnsiStyle, Color};
            let cyan = AnsiStyle::new(Color::Cyan, false);

            match state {
                DirenvState::Allowed => Some(format!(
                    "{}[{}]{} ",
                    cyan.start_codes(),
                    text,
                    AnsiStyle::RESET
                )),
                DirenvState::Blocked => {
                    let bold_red = AnsiStyle::new(Color::Red, true);
                    Some(format!(
                        "{}[{}{}{}]{} ",
                        cyan.start_codes(),
                        bold_red.start_codes(),
                        text,
                        cyan.start_codes(),
                        AnsiStyle::RESET,
                    ))
                }
            }
        }
    }
}

enum DirenvState {
    Allowed,
    Blocked,
}

impl Module for DirenvModule {
    fn render(&self, context: &ModuleContext) -> Result<Option<String>> {
        let direnv_file = utils::find_upward(".envrc");
        Ok(Self::render_with_direnv_file(
            direnv_file.as_deref(),
            context,
        ))
    }
}

#[derive(Deserialize)]
struct DirenvStatusStateResult {
    allowed: u8,
}

#[derive(Deserialize)]
struct DirenvStatusState {
    #[serde(rename = "loadedRC")]
    loaded_rc: DirenvStatusStateResult,
}

#[derive(Deserialize)]
struct DirenvStatus {
    state: DirenvStatusState,
}

fn get_direnv_status_slow(direnv_root: &Path) -> Option<DirenvState> {
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

    let status_text = String::from_utf8_lossy(&output.stdout);
    let Ok(status) = serde_json::from_str::<DirenvStatus>(&status_text) else {
        return None; // JSON parsing failed
    };

    // Check if direnv is loaded (direnv uses 0=loaded, 1=blocked)
    if status.state.loaded_rc.allowed == 0 {
        Some(DirenvState::Allowed)
    } else {
        Some(DirenvState::Blocked)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_direnv_without_envrc() {
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
}
