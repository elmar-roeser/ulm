//! Manpage scanning and indexing.
//!
//! This module scans system directories to find all available manpages
//! and prepares them for embedding generation.

use std::env;
use std::fs;
use std::path::{Path, PathBuf};

use anyhow::{Context, Result};
use tracing::{debug, info, warn};

/// Default manpage directories to scan.
const DEFAULT_PATHS: &[&str] = &[
    "/usr/share/man",
    "/usr/local/share/man",
    "/opt/homebrew/share/man", // macOS Homebrew
];

/// Manpage sections to scan (user commands and system administration).
const SECTIONS: &[&str] = &["man1", "man8"];

/// Scanner for finding manpage files on the system.
#[derive(Debug)]
pub struct ManpageScanner {
    /// Directories to scan for manpages.
    paths: Vec<PathBuf>,
}

impl ManpageScanner {
    /// Creates a new scanner with default paths and $MANPATH.
    #[must_use]
    pub fn new() -> Self {
        let mut paths: Vec<PathBuf> = DEFAULT_PATHS.iter().map(PathBuf::from).collect();

        // Add paths from $MANPATH
        if let Ok(manpath) = env::var("MANPATH") {
            for path in manpath.split(':') {
                if !path.is_empty() {
                    let path_buf = PathBuf::from(path);
                    if !paths.contains(&path_buf) {
                        paths.push(path_buf);
                    }
                }
            }
        }

        debug!(?paths, "Initialized manpage scanner");
        Self { paths }
    }

    /// Creates a scanner with custom paths (for testing).
    #[must_use]
    pub const fn with_paths(paths: Vec<PathBuf>) -> Self {
        Self { paths }
    }

    /// Scans all configured directories for manpage files.
    ///
    /// Returns a list of paths to manpage files (man1 and man8 sections).
    ///
    /// # Errors
    ///
    /// Returns an error if directory reading fails unexpectedly.
    pub fn scan_directories(&self) -> Result<Vec<PathBuf>> {
        let mut manpages = Vec::new();

        for base_path in &self.paths {
            if !base_path.exists() {
                debug!(?base_path, "Manpage directory does not exist, skipping");
                continue;
            }

            for section in SECTIONS {
                let section_path = base_path.join(section);
                if !section_path.exists() {
                    debug!(?section_path, "Section directory does not exist, skipping");
                    continue;
                }

                match Self::scan_section(&section_path) {
                    Ok(pages) => {
                        debug!(
                            path = ?section_path,
                            count = pages.len(),
                            "Scanned section"
                        );
                        manpages.extend(pages);
                    }
                    Err(e) => {
                        warn!(path = ?section_path, error = %e, "Failed to scan section");
                    }
                }
            }
        }

        info!(count = manpages.len(), "Total manpages found");
        Ok(manpages)
    }

    /// Scans a single section directory for manpage files.
    fn scan_section(section_path: &Path) -> Result<Vec<PathBuf>> {
        let mut pages = Vec::new();

        let entries = fs::read_dir(section_path)
            .with_context(|| format!("Failed to read directory: {}", section_path.display()))?;

        for entry in entries {
            let entry = entry.with_context(|| {
                format!("Failed to read entry in: {}", section_path.display())
            })?;

            let path = entry.path();
            if Self::is_manpage_file(&path) {
                pages.push(path);
            }
        }

        Ok(pages)
    }

    /// Checks if a file is a manpage based on its extension.
    fn is_manpage_file(path: &Path) -> bool {
        let Some(name) = path.file_name().and_then(|n| n.to_str()) else {
            return false;
        };

        // Check for .1, .8, .1.gz, .8.gz extensions
        name.ends_with(".1")
            || name.ends_with(".8")
            || name.ends_with(".1.gz")
            || name.ends_with(".8.gz")
    }

    /// Returns the configured paths.
    #[must_use]
    pub fn paths(&self) -> &[PathBuf] {
        &self.paths
    }
}

impl Default for ManpageScanner {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs::File;
    use tempfile::TempDir;

    fn create_test_structure(temp_dir: &TempDir) -> PathBuf {
        let base = temp_dir.path().to_path_buf();

        // Create man1 section
        let man1 = base.join("man1");
        fs::create_dir_all(&man1).unwrap();
        File::create(man1.join("ls.1")).unwrap();
        File::create(man1.join("cat.1.gz")).unwrap();
        File::create(man1.join("readme.txt")).unwrap(); // Should be ignored

        // Create man8 section
        let man8 = base.join("man8");
        fs::create_dir_all(&man8).unwrap();
        File::create(man8.join("mount.8")).unwrap();
        File::create(man8.join("fsck.8.gz")).unwrap();

        base
    }

    #[test]
    fn test_scanner_creation() {
        let scanner = ManpageScanner::new();
        assert!(!scanner.paths().is_empty());
    }

    #[test]
    fn test_scanner_with_custom_paths() {
        let paths = vec![PathBuf::from("/custom/path")];
        let scanner = ManpageScanner::with_paths(paths.clone());
        assert_eq!(scanner.paths(), &paths);
    }

    #[test]
    fn test_scan_test_directory() {
        let temp_dir = TempDir::new().unwrap();
        let base = create_test_structure(&temp_dir);

        let scanner = ManpageScanner::with_paths(vec![base]);
        let pages = scanner.scan_directories().unwrap();

        // Should find 4 manpages (2 in man1, 2 in man8)
        assert_eq!(pages.len(), 4);
    }

    #[test]
    fn test_ignores_non_manpage_files() {
        let temp_dir = TempDir::new().unwrap();
        let base = create_test_structure(&temp_dir);

        let scanner = ManpageScanner::with_paths(vec![base]);
        let pages = scanner.scan_directories().unwrap();

        // readme.txt should not be included
        assert!(!pages.iter().any(|p| p.to_string_lossy().contains("readme")));
    }

    #[test]
    fn test_handles_missing_directories() {
        let scanner = ManpageScanner::with_paths(vec![PathBuf::from("/nonexistent/path")]);
        let pages = scanner.scan_directories().unwrap();
        assert!(pages.is_empty());
    }

    #[test]
    fn test_is_manpage_file() {
        assert!(ManpageScanner::is_manpage_file(Path::new("ls.1")));
        assert!(ManpageScanner::is_manpage_file(Path::new("cat.1.gz")));
        assert!(ManpageScanner::is_manpage_file(Path::new("mount.8")));
        assert!(ManpageScanner::is_manpage_file(Path::new("fsck.8.gz")));

        assert!(!ManpageScanner::is_manpage_file(Path::new("readme.txt")));
        assert!(!ManpageScanner::is_manpage_file(Path::new("lib.3"))); // man3 not supported
        assert!(!ManpageScanner::is_manpage_file(Path::new("config.5"))); // man5 not supported
    }
}
