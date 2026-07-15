use std::env;
use std::path::{Path, PathBuf};

#[must_use]
pub fn find_upward(name: &str) -> Option<PathBuf> {
    let current_dir = env::current_dir().ok()?;
    find_upward_from(&current_dir, name)
}

#[must_use]
pub fn find_upward_from(start_dir: &Path, name: &str) -> Option<PathBuf> {
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
}
