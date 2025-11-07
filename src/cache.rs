use std::collections::HashMap;
use std::path::{Path, PathBuf};
use std::sync::RwLock;
use std::time::{Duration, Instant};

pub struct CacheEntry<T> {
    value: T,
    timestamp: Instant,
    ttl: Duration,
}

impl<T: Clone> CacheEntry<T> {
    #[must_use]
    fn new(value: T, ttl: Duration) -> Self {
        Self {
            value,
            timestamp: Instant::now(),
            ttl,
        }
    }

    fn is_valid(&self) -> bool {
        self.timestamp.elapsed() < self.ttl
    }

    fn get(&self) -> Option<T> {
        if self.is_valid() {
            Some(self.value.clone())
        } else {
            None
        }
    }
}

pub struct GitCache {
    entries: RwLock<HashMap<PathBuf, CacheEntry<GitInfo>>>,
}

#[derive(Clone)]
pub struct GitInfo {
    pub branch: String,
    pub has_changes: bool,
    pub has_untracked: bool,
}

impl Default for GitCache {
    fn default() -> Self {
        Self::new()
    }
}

impl GitCache {
    #[must_use]
    pub fn new() -> Self {
        Self {
            entries: RwLock::new(HashMap::new()),
        }
    }

    pub fn get(&self, path: &Path) -> Option<GitInfo> {
        let entries = self.entries.read().ok()?;
        entries.get(path)?.get()
    }

    pub fn insert(&self, path: PathBuf, info: GitInfo) {
        if let Ok(mut entries) = self.entries.write() {
            // Git info cached for 1 second (frequent prompt refreshes)
            entries.insert(path, CacheEntry::new(info, Duration::from_millis(1000)));
        }
    }
}

pub static GIT_CACHE: std::sync::LazyLock<GitCache> = std::sync::LazyLock::new(GitCache::new);
