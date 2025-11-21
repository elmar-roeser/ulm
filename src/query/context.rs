//! Directory context scanning for project awareness.
//!
//! This module detects the project type and relevant marker files in the
//! current working directory to provide context-aware command suggestions.

use std::env;
use std::path::{Path, PathBuf};

use anyhow::{Context, Result};
use tracing::debug;

/// Detected project type based on marker files.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ProjectType {
    /// Rust project (Cargo.toml).
    Rust,
    /// Node.js project (package.json).
    Node,
    /// Python project (pyproject.toml, requirements.txt).
    Python,
    /// Go project (go.mod).
    Go,
    /// `CMake` project (CMakeLists.txt).
    CMake,
    /// Git repository (.git).
    Git,
    /// Unknown project type.
    Unknown,
}

impl ProjectType {
    /// Returns a human-readable name for the project type.
    #[must_use]
    pub const fn as_str(&self) -> &'static str {
        match self {
            Self::Rust => "Rust",
            Self::Node => "Node.js",
            Self::Python => "Python",
            Self::Go => "Go",
            Self::CMake => "CMake",
            Self::Git => "Git",
            Self::Unknown => "Unknown",
        }
    }
}

/// Context information about the current directory.
#[derive(Debug, Clone)]
pub struct DirectoryContext {
    /// Detected primary project type.
    pub project_type: Option<ProjectType>,
    /// List of detected marker files.
    pub marker_files: Vec<String>,
    /// Current working directory.
    pub cwd: PathBuf,
}

/// Marker files and their associated project types.
/// Order determines detection priority.
const MARKER_FILES: &[(&str, ProjectType)] = &[
    ("Cargo.toml", ProjectType::Rust),
    ("package.json", ProjectType::Node),
    ("go.mod", ProjectType::Go),
    ("pyproject.toml", ProjectType::Python),
    ("requirements.txt", ProjectType::Python),
    ("CMakeLists.txt", ProjectType::CMake),
    (".git", ProjectType::Git),
];

/// Scans the current directory for project context.
///
/// Detects the project type by checking for marker files in the top-level
/// directory only (no recursion). Returns context information including
/// the detected project type and list of marker files found.
///
/// # Errors
///
/// Returns an error if:
/// - Cannot determine current working directory
/// - Cannot read directory contents
pub fn scan_directory_context() -> Result<DirectoryContext> {
    let cwd = env::current_dir().context("Failed to get current directory")?;
    scan_directory_context_at(&cwd)
}

/// Scans a specific directory for project context.
///
/// Internal function that allows scanning any directory path.
fn scan_directory_context_at(path: &Path) -> Result<DirectoryContext> {
    debug!(path = %path.display(), "Scanning directory context");

    let mut marker_files = Vec::new();
    let mut project_type: Option<ProjectType> = None;

    // Read directory entries (top-level only)
    let entries = std::fs::read_dir(path)
        .with_context(|| format!("Failed to read directory: {}", path.display()))?;

    // Collect all filenames
    let filenames: Vec<String> = entries
        .flatten()
        .filter_map(|e| e.file_name().to_str().map(String::from))
        .collect();

    // Check for marker files in priority order
    for (marker, ptype) in MARKER_FILES {
        if filenames.contains(&(*marker).to_string()) {
            marker_files.push((*marker).to_string());

            // Set project type only if not already set (first match wins)
            if project_type.is_none() {
                project_type = Some(ptype.clone());
                debug!(
                    marker = %marker,
                    project_type = ptype.as_str(),
                    "Detected project type"
                );
            }
        }
    }

    debug!(
        markers = marker_files.len(),
        project_type = project_type.as_ref().map_or("None", |p| p.as_str()),
        "Directory context scan complete"
    );

    Ok(DirectoryContext {
        project_type,
        marker_files,
        cwd: path.to_path_buf(),
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs::{self, File};
    use tempfile::TempDir;

    #[test]
    fn test_project_type_as_str() {
        assert_eq!(ProjectType::Rust.as_str(), "Rust");
        assert_eq!(ProjectType::Node.as_str(), "Node.js");
        assert_eq!(ProjectType::Python.as_str(), "Python");
        assert_eq!(ProjectType::Go.as_str(), "Go");
        assert_eq!(ProjectType::CMake.as_str(), "CMake");
        assert_eq!(ProjectType::Git.as_str(), "Git");
        assert_eq!(ProjectType::Unknown.as_str(), "Unknown");
    }

    #[test]
    fn test_scan_rust_project() {
        let temp_dir = TempDir::new().unwrap();
        File::create(temp_dir.path().join("Cargo.toml")).unwrap();

        let context = scan_directory_context_at(temp_dir.path()).unwrap();
        assert_eq!(context.project_type, Some(ProjectType::Rust));
        assert!(context.marker_files.contains(&"Cargo.toml".to_string()));
    }

    #[test]
    fn test_scan_node_project() {
        let temp_dir = TempDir::new().unwrap();
        File::create(temp_dir.path().join("package.json")).unwrap();

        let context = scan_directory_context_at(temp_dir.path()).unwrap();
        assert_eq!(context.project_type, Some(ProjectType::Node));
        assert!(context.marker_files.contains(&"package.json".to_string()));
    }

    #[test]
    fn test_scan_python_project() {
        let temp_dir = TempDir::new().unwrap();
        File::create(temp_dir.path().join("requirements.txt")).unwrap();

        let context = scan_directory_context_at(temp_dir.path()).unwrap();
        assert_eq!(context.project_type, Some(ProjectType::Python));
        assert!(context
            .marker_files
            .contains(&"requirements.txt".to_string()));
    }

    #[test]
    fn test_scan_go_project() {
        let temp_dir = TempDir::new().unwrap();
        File::create(temp_dir.path().join("go.mod")).unwrap();

        let context = scan_directory_context_at(temp_dir.path()).unwrap();
        assert_eq!(context.project_type, Some(ProjectType::Go));
        assert!(context.marker_files.contains(&"go.mod".to_string()));
    }

    #[test]
    fn test_scan_git_repo() {
        let temp_dir = TempDir::new().unwrap();
        fs::create_dir(temp_dir.path().join(".git")).unwrap();

        let context = scan_directory_context_at(temp_dir.path()).unwrap();
        assert_eq!(context.project_type, Some(ProjectType::Git));
        assert!(context.marker_files.contains(&".git".to_string()));
    }

    #[test]
    fn test_scan_empty_directory() {
        let temp_dir = TempDir::new().unwrap();

        let context = scan_directory_context_at(temp_dir.path()).unwrap();
        assert_eq!(context.project_type, None);
        assert!(context.marker_files.is_empty());
    }

    #[test]
    fn test_priority_rust_over_git() {
        let temp_dir = TempDir::new().unwrap();
        File::create(temp_dir.path().join("Cargo.toml")).unwrap();
        fs::create_dir(temp_dir.path().join(".git")).unwrap();

        let context = scan_directory_context_at(temp_dir.path()).unwrap();
        // Rust should win over Git due to priority
        assert_eq!(context.project_type, Some(ProjectType::Rust));
        assert!(context.marker_files.contains(&"Cargo.toml".to_string()));
        assert!(context.marker_files.contains(&".git".to_string()));
    }

    #[test]
    fn test_multiple_markers() {
        let temp_dir = TempDir::new().unwrap();
        File::create(temp_dir.path().join("package.json")).unwrap();
        File::create(temp_dir.path().join("requirements.txt")).unwrap();

        let context = scan_directory_context_at(temp_dir.path()).unwrap();
        // Node should win due to priority
        assert_eq!(context.project_type, Some(ProjectType::Node));
        assert_eq!(context.marker_files.len(), 2);
    }
}
