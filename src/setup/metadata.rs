//! Metadata tracking for incremental updates.
//!
//! Stores file hashes to enable processing only new/changed manpages.

use std::collections::HashMap;
use std::fs;
use std::io::{BufReader, Read};
use std::path::{Path, PathBuf};

use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};
use tracing::{debug, info};

use crate::db;

/// Metadata for indexed manpages.
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct IndexMetadata {
    /// Map of file path to content hash.
    pub files: HashMap<String, String>,
}

impl IndexMetadata {
    /// Loads metadata from disk, or returns empty if not found.
    ///
    /// # Errors
    ///
    /// Returns an error if the file exists but cannot be read or parsed.
    pub fn load() -> Result<Self> {
        let path = get_metadata_path()?;

        if !path.exists() {
            return Ok(Self::default());
        }

        let content = fs::read_to_string(&path)
            .with_context(|| format!("Failed to read metadata: {}", path.display()))?;

        let metadata: Self = serde_json::from_str(&content)
            .with_context(|| format!("Failed to parse metadata: {}", path.display()))?;

        info!(files = metadata.files.len(), "Loaded index metadata");
        Ok(metadata)
    }

    /// Saves metadata to disk.
    ///
    /// # Errors
    ///
    /// Returns an error if the file cannot be written.
    pub fn save(&self) -> Result<()> {
        let path = get_metadata_path()?;

        // Ensure parent directory exists
        if let Some(parent) = path.parent() {
            fs::create_dir_all(parent).with_context(|| {
                format!("Failed to create metadata directory: {}", parent.display())
            })?;
        }

        let content = serde_json::to_string_pretty(self).context("Failed to serialize metadata")?;

        fs::write(&path, content)
            .with_context(|| format!("Failed to write metadata: {}", path.display()))?;

        info!(files = self.files.len(), "Saved index metadata");
        Ok(())
    }

    /// Filters paths to only those that are new or changed.
    ///
    /// Returns tuple of (paths to process, count of unchanged).
    pub fn filter_changed(&self, paths: Vec<PathBuf>) -> (Vec<PathBuf>, usize) {
        let mut to_process = Vec::new();
        let mut unchanged = 0;

        for path in paths {
            let path_str = path.to_string_lossy().to_string();

            match compute_file_hash(&path) {
                Ok(hash) => {
                    if let Some(stored_hash) = self.files.get(&path_str) {
                        if &hash == stored_hash {
                            unchanged += 1;
                            continue;
                        }
                        debug!(path = %path_str, "File changed");
                    } else {
                        debug!(path = %path_str, "New file");
                    }
                    to_process.push(path);
                }
                Err(e) => {
                    debug!(path = %path_str, error = %e, "Failed to hash file");
                    to_process.push(path);
                }
            }
        }

        (to_process, unchanged)
    }

    /// Updates metadata with new file hashes.
    pub fn update_hashes(&mut self, paths: &[PathBuf]) {
        for path in paths {
            let path_str = path.to_string_lossy().to_string();

            if let Ok(hash) = compute_file_hash(path) {
                self.files.insert(path_str, hash);
            }
        }
    }

    /// Removes entries for files that no longer exist.
    pub fn remove_deleted(&mut self) {
        let before = self.files.len();
        self.files.retain(|path, _| Path::new(path).exists());
        let removed = before - self.files.len();

        if removed > 0 {
            info!(removed = removed, "Removed deleted files from metadata");
        }
    }
}

/// Gets the path to the metadata file.
fn get_metadata_path() -> Result<PathBuf> {
    let db_path = db::get_database_path()?;
    let parent = db_path.parent().context("Database path has no parent")?;

    Ok(parent.join("index_metadata.json"))
}

/// Computes a hash of the file contents.
fn compute_file_hash(path: &Path) -> Result<String> {
    let file =
        fs::File::open(path).with_context(|| format!("Failed to open file: {}", path.display()))?;

    let mut reader = BufReader::new(file);
    let mut hasher = blake3::Hasher::new();
    let mut buffer = [0; 8192];

    loop {
        let bytes_read = reader
            .read(&mut buffer)
            .with_context(|| format!("Failed to read file: {}", path.display()))?;

        if bytes_read == 0 {
            break;
        }

        hasher.update(&buffer[..bytes_read]);
    }

    Ok(hasher.finalize().to_hex().to_string())
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    #[test]
    fn test_metadata_default() {
        let metadata = IndexMetadata::default();
        assert!(metadata.files.is_empty());
    }

    #[test]
    fn test_compute_hash() {
        let temp = TempDir::new().unwrap();
        let file_path = temp.path().join("test.txt");
        fs::write(&file_path, "test content").unwrap();

        let hash1 = compute_file_hash(&file_path).unwrap();
        let hash2 = compute_file_hash(&file_path).unwrap();

        assert_eq!(hash1, hash2);
        assert!(!hash1.is_empty());
    }

    #[test]
    fn test_hash_changes_with_content() {
        let temp = TempDir::new().unwrap();
        let file_path = temp.path().join("test.txt");

        fs::write(&file_path, "content 1").unwrap();
        let hash1 = compute_file_hash(&file_path).unwrap();

        fs::write(&file_path, "content 2").unwrap();
        let hash2 = compute_file_hash(&file_path).unwrap();

        assert_ne!(hash1, hash2);
    }

    #[test]
    fn test_filter_changed() {
        let temp = TempDir::new().unwrap();
        let file1 = temp.path().join("file1.txt");
        let file2 = temp.path().join("file2.txt");

        fs::write(&file1, "content 1").unwrap();
        fs::write(&file2, "content 2").unwrap();

        let mut metadata = IndexMetadata::default();
        metadata.update_hashes(&[file1.clone()]);

        let paths = vec![file1.clone(), file2.clone()];
        let (to_process, unchanged) = metadata.filter_changed(paths);

        assert_eq!(unchanged, 1);
        assert_eq!(to_process.len(), 1);
        assert_eq!(to_process[0], file2);
    }
}
