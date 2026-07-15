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
            return result;
        }
    }

    current_dir.to_string_lossy().to_string()
}

impl Module for PathModule {
    fn render(&self, context: &ModuleContext) -> Result<Option<String>> {
        let Ok(current_dir) = env::current_dir() else {
            return Ok(None);
        };

        let path = normalize_relative_path(&current_dir);

        if context.no_color {
            Ok(Some(format!("{path} ")))
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

    #[test]
    fn test_path_renders() {
        let module = PathModule::new();
        let context = ModuleContext::default();

        let result = module.render(&context).unwrap();
        assert!(result.is_some(), "Path module should render something");

        let output = result.unwrap();
        assert!(!output.is_empty());
        assert!(output.ends_with(' '), "Should have trailing space");
    }

    #[test]
    fn path_inside_home_uses_tilde_with_native_separator() {
        let home = dirs::home_dir().expect("home dir should exist");
        let temp_dir = tempfile::tempdir_in(&home).expect("create temp dir in home");
        let name = temp_dir.path().file_name().expect("temp dir name");
        let expected = Path::new("~").join(name).to_string_lossy().into_owned();

        assert_eq!(normalize_relative_path(temp_dir.path()), expected);
    }

    #[test]
    fn path_with_a_home_prefix_is_not_rendered_as_home() {
        let home = dirs::home_dir().expect("home dir should exist");
        let parent = home.parent().expect("home parent");
        let home_name = home.file_name().expect("home name").to_string_lossy();
        let sibling = parent.join(format!("{home_name}-similar"));
        let expected = sibling.to_string_lossy().into_owned();

        assert_eq!(normalize_relative_path(&sibling), expected);
    }
}
