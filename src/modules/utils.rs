use std::borrow::Cow;
use std::env;
use std::path::{Path, PathBuf};

/// Escapes controls before dynamic text is interpolated into prompt output.
/// Printable Unicode is preserved, while controls use visible Rust-style escapes.
pub(crate) fn sanitize_display_text(text: &str) -> Cow<'_, str> {
    if !text.chars().any(char::is_control) {
        return Cow::Borrowed(text);
    }

    let mut sanitized = String::with_capacity(text.len());
    for character in text.chars() {
        if character.is_control() {
            sanitized.extend(character.escape_debug());
        } else {
            sanitized.push(character);
        }
    }

    Cow::Owned(sanitized)
}

#[must_use]
pub(crate) fn find_upward(name: &str) -> Option<PathBuf> {
    let current_dir = env::current_dir().ok()?;
    find_upward_from(&current_dir, name)
}

#[must_use]
fn find_upward_from(start_dir: &Path, name: &str) -> Option<PathBuf> {
    let mut current = start_dir.to_path_buf();

    loop {
        let potential = current.join(name);
        if potential.exists() {
            return Some(potential);
        }

        let parent = current.parent()?;
        current = parent.to_path_buf();
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::borrow::Cow;
    use std::fs;

    #[test]
    fn finds_file_in_ancestor_directory() {
        let temp_dir = tempfile::tempdir().expect("create temp dir");
        let ancestor = temp_dir.path().join("ancestor");
        let descendant = ancestor.join("descendant");
        let marker = ancestor.join(".envrc");

        fs::create_dir_all(&descendant).expect("create descendant directory");
        fs::write(&marker, "").expect("create marker file");

        assert_eq!(find_upward_from(&descendant, ".envrc"), Some(marker));
    }

    #[test]
    fn returns_none_when_file_is_absent() {
        let temp_dir = tempfile::tempdir().expect("create temp dir");
        let descendant = temp_dir.path().join("ancestor/descendant");

        fs::create_dir_all(&descendant).expect("create descendant directory");

        assert_eq!(find_upward_from(&descendant, ".envrc"), None);
    }

    #[test]
    fn sanitize_display_text_preserves_printable_unicode_without_allocating() {
        let text = "plain café 🦀";

        assert!(matches!(
            sanitize_display_text(text),
            Cow::Borrowed(sanitized) if sanitized == text
        ));
    }

    #[test]
    fn sanitize_display_text_uses_rust_style_escapes_for_controls() {
        let text = "nul:\0 tab:\t line:\n carriage:\r escape:\u{1b} delete:\u{7f} c1:\u{85}";
        let expected = r"nul:\0 tab:\t line:\n carriage:\r escape:\u{1b} delete:\u{7f} c1:\u{85}";

        assert_eq!(sanitize_display_text(text), expected);
    }
}
