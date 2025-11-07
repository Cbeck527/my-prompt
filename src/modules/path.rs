use crate::error::Result;
use crate::module_trait::{Module, ModuleContext};
use std::env;
use std::path::Path;

pub struct PathModule;

impl Default for PathModule {
    fn default() -> Self {
        Self::new()
    }
}

impl PathModule {
    #[must_use]
    pub fn new() -> Self {
        Self
    }
}

#[cfg(target_os = "windows")]
fn normalize_separators(value: String) -> String {
    value.replace('\\', "/")
}

#[cfg(not(target_os = "windows"))]
fn normalize_separators(value: String) -> String {
    value
}

fn normalize_relative_path(current_dir: &Path) -> String {
    let current_canon = current_dir
        .canonicalize()
        .unwrap_or_else(|_| current_dir.to_path_buf());

    if let Some(home) = dirs::home_dir() {
        let home_canon = home.canonicalize().unwrap_or(home);
        if let Ok(stripped) = current_canon.strip_prefix(&home_canon) {
            if stripped.as_os_str().is_empty() {
                return "~".to_string();
            }

            let mut result = String::from("~");
            result.push(std::path::MAIN_SEPARATOR);
            result.push_str(&stripped.to_string_lossy());
            return normalize_separators(result);
        }
    }

    normalize_separators(current_dir.to_string_lossy().to_string())
}

impl Module for PathModule {
    fn render(&self, context: &ModuleContext) -> Result<Option<String>> {
        let Ok(current_dir) = env::current_dir() else {
            return Ok(None);
        };

        let path = normalize_relative_path(&current_dir);

        if context.no_color {
            Ok(Some(format!("{} ", path)))
        } else {
            use crate::style::{AnsiStyle, Color};
            let style = AnsiStyle::new(Color::White, false);
            Ok(Some(format!(
                "{}{}{} ",
                style.start_codes(),
                path,
                AnsiStyle::RESET
            )))
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serial_test::serial;
    use std::fs;
    use std::time::{SystemTime, UNIX_EPOCH};

    struct DirGuard {
        original: std::path::PathBuf,
    }

    impl DirGuard {
        fn change_to(path: &Path) -> Self {
            let original = env::current_dir().expect("current dir");
            env::set_current_dir(path).expect("change current dir");
            Self { original }
        }
    }

    impl Drop for DirGuard {
        fn drop(&mut self) {
            let _ = env::set_current_dir(&self.original);
        }
    }

    fn unique_name() -> String {
        SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .expect("system time")
            .as_nanos()
            .to_string()
    }

    #[test]
    #[serial]
    fn relative_path_inside_home_renders_tilde() {
        let module = PathModule::new();
        let home = dirs::home_dir().expect("home dir should exist");
        let project = home.join(format!("prmt_test_project_{}", unique_name()));
        fs::create_dir_all(&project).expect("create project dir");

        let _dir_guard = DirGuard::change_to(&project);

        let value = module
            .render(&ModuleContext::default())
            .expect("render")
            .expect("some");

        // Should have trailing space
        assert!(
            value.ends_with(" "),
            "Expected trailing space, got: {}",
            value
        );

        // Strip trailing space for path check
        let path = value.trim_end();
        assert!(
            path.starts_with("~/prmt_test_project_"),
            "Expected path to start with ~/prmt_test_project_, got: {}",
            path
        );

        let _ = fs::remove_dir_all(&project);
    }

    #[test]
    #[serial]
    fn relative_path_with_shared_prefix_is_not_tilde() {
        let module = PathModule::new();
        let home = dirs::home_dir().expect("home dir should exist");

        let unique = unique_name();
        let base = home.join(format!("prmt_test_base_{}", unique));
        let home_like = base.join("al");
        let similar = base.join("alpine");

        fs::create_dir_all(&home_like).expect("create home_like");
        fs::create_dir_all(&similar).expect("create similar");

        let _dir_guard = DirGuard::change_to(&similar);

        let value = module
            .render(&ModuleContext::default())
            .expect("render")
            .expect("some");

        // Strip trailing space for path check
        let path = value.trim_end();
        assert!(
            path.starts_with("~/prmt_test_base_"),
            "Expected path to start with ~/prmt_test_base_, got: {}",
            path
        );
        assert!(
            path.ends_with("/alpine"),
            "Expected path to end with /alpine, got: {}",
            path
        );

        let _ = fs::remove_dir_all(&base);
    }
}
