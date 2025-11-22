//! Manpage scanning and indexing.
//!
//! This module scans system directories to find all available manpages
//! and prepares them for embedding generation.

use std::env;
use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;
use std::time::Duration;

use anyhow::{Context, Result};
use tokio::time::sleep;
use tracing::{debug, info, warn};

use crate::llm::{OllamaClient, EMBEDDING_MODEL};

/// Extracted content from a manpage.
#[derive(Debug, Clone)]
pub struct ManpageContent {
    /// Tool name (e.g., "ls").
    pub tool_name: String,
    /// Section number (e.g., "1").
    pub section: String,
    /// Combined NAME and DESCRIPTION text for embedding.
    pub description: String,
}

/// Manpage entry with embedding vector for storage.
#[derive(Debug, Clone)]
pub struct ManpageEntry {
    /// Tool name (e.g., "ls").
    pub tool_name: String,
    /// Section number (e.g., "1").
    pub section: String,
    /// Combined NAME and DESCRIPTION text.
    pub description: String,
    /// Embedding vector.
    pub vector: Vec<f32>,
}

/// Generator for creating embeddings from manpage content.
#[derive(Debug)]
pub struct EmbeddingGenerator {
    /// Ollama client for API calls.
    client: OllamaClient,
    /// Model to use for embeddings.
    model: String,
}

impl EmbeddingGenerator {
    /// Creates a new embedding generator with default model.
    ///
    /// # Errors
    ///
    /// Returns an error if the client cannot be created.
    pub fn new() -> Result<Self> {
        let client = OllamaClient::new()?;
        Ok(Self {
            client,
            model: EMBEDDING_MODEL.to_string(),
        })
    }

    /// Creates a new embedding generator with custom client and model.
    #[must_use]
    pub fn with_client(client: OllamaClient, model: &str) -> Self {
        Self {
            client,
            model: model.to_string(),
        }
    }

    /// Generates embeddings for a list of manpage contents.
    ///
    /// Processes in batches with progress display and retry logic.
    ///
    /// # Errors
    ///
    /// Returns an error if embedding generation fails for any item after retries.
    pub async fn generate_embeddings(
        &self,
        contents: Vec<ManpageContent>,
    ) -> Result<Vec<ManpageEntry>> {
        let total = contents.len();
        let mut entries = Vec::with_capacity(total);
        let batch_size = 10;

        info!(total = total, "Starting embedding generation");

        for (i, content) in contents.into_iter().enumerate() {
            // Progress display
            if i % batch_size == 0 || i == total - 1 {
                println!("Generating embeddings... {}/{}", i + 1, total);
            }

            // Generate embedding with retry
            let vector = self.generate_with_retry(&content.description).await?;

            entries.push(ManpageEntry {
                tool_name: content.tool_name,
                section: content.section,
                description: content.description,
                vector,
            });
        }

        info!(count = entries.len(), "Embedding generation complete");
        Ok(entries)
    }

    /// Generates embedding with retry logic.
    async fn generate_with_retry(&self, text: &str) -> Result<Vec<f32>> {
        let max_attempts = 3;

        for attempt in 1..=max_attempts {
            match self.client.generate_embedding(&self.model, text).await {
                Ok(vector) => return Ok(vector),
                Err(e) if attempt < max_attempts => {
                    let delay = Duration::from_secs(2_u64.pow(attempt));
                    warn!(
                        attempt = attempt,
                        delay_secs = delay.as_secs(),
                        error = %e,
                        "Embedding generation failed, retrying"
                    );
                    sleep(delay).await;
                }
                Err(e) => {
                    return Err(e).context("Embedding generation failed after 3 attempts");
                }
            }
        }

        unreachable!()
    }
}

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
            let entry = entry
                .with_context(|| format!("Failed to read entry in: {}", section_path.display()))?;

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

    /// Extracts content from a manpage file.
    ///
    /// # Errors
    ///
    /// Returns an error if the manpage cannot be read or parsed.
    pub fn extract_content(path: &Path) -> Result<ManpageContent> {
        let (tool_name, section) = Self::parse_filename(path)?;

        debug!(tool = %tool_name, section = %section, "Extracting manpage content");

        // Run man -P cat to get raw content
        let output = Command::new("man")
            .args(["-P", "cat", &tool_name])
            .output()
            .with_context(|| format!("Failed to execute man command for '{tool_name}'"))?;

        if !output.status.success() {
            let stderr = String::from_utf8_lossy(&output.stderr);
            anyhow::bail!("man command failed for '{}': {}", tool_name, stderr.trim());
        }

        // Convert output to UTF-8
        let content = String::from_utf8(output.stdout)
            .with_context(|| format!("Manpage '{tool_name}' contains invalid UTF-8"))?;

        // Parse NAME and DESCRIPTION
        let description = Self::parse_manpage_content(&content, &tool_name);

        Ok(ManpageContent {
            tool_name,
            section,
            description,
        })
    }

    /// Parses filename to extract tool name and section.
    fn parse_filename(path: &Path) -> Result<(String, String)> {
        let filename = path
            .file_name()
            .and_then(|n| n.to_str())
            .context("Invalid manpage filename")?;

        // Remove .gz if present
        let filename = filename.strip_suffix(".gz").unwrap_or(filename);

        // Extract section (last character before extension)
        let section = filename
            .chars()
            .last()
            .map_or_else(|| "1".to_string(), |c| c.to_string());

        // Extract tool name (everything before the dot and section)
        let tool_name = filename
            .rsplit_once('.')
            .map_or_else(|| filename.to_string(), |(name, _)| name.to_string());

        Ok((tool_name, section))
    }

    /// Parses manpage content to extract NAME and DESCRIPTION.
    fn parse_manpage_content(content: &str, tool_name: &str) -> String {
        let mut result = String::new();

        // Try to find NAME section
        if let Some(name_text) = Self::extract_section(content, "NAME") {
            // Take first line of NAME
            let first_line = name_text.lines().next().unwrap_or("").trim();
            if !first_line.is_empty() {
                result.push_str(first_line);
            }
        }

        // If no NAME found, use tool name
        if result.is_empty() {
            result.push_str(tool_name);
        }

        // Try to find DESCRIPTION section
        if let Some(desc_text) = Self::extract_section(content, "DESCRIPTION") {
            // Take first paragraph (up to 500 chars)
            let first_para = Self::extract_first_paragraph(&desc_text);
            if !first_para.is_empty() {
                if !result.is_empty() {
                    result.push_str(" - ");
                }
                result.push_str(&first_para);
            }
        }

        // Limit total length (handle UTF-8 boundaries)
        if result.len() > 500 {
            // Find last valid char boundary at or before 500
            let mut end = 500;
            while !result.is_char_boundary(end) && end > 0 {
                end -= 1;
            }
            result.truncate(end);
            result.push_str("...");
        }

        result
    }

    /// Extracts a section from manpage content.
    fn extract_section(content: &str, section_name: &str) -> Option<String> {
        let lines: Vec<&str> = content.lines().collect();
        let mut in_section = false;
        let mut section_text = String::new();

        for line in lines {
            let trimmed = line.trim();

            // Check if this is a section header
            if trimmed == section_name || trimmed == section_name.to_uppercase() {
                in_section = true;
                continue;
            }

            // Check if we've hit the next section (all caps line)
            if in_section
                && !trimmed.is_empty()
                && trimmed
                    .chars()
                    .all(|c| c.is_uppercase() || c.is_whitespace())
                && trimmed.len() > 2
            {
                break;
            }

            if in_section && !trimmed.is_empty() {
                if !section_text.is_empty() {
                    section_text.push(' ');
                }
                section_text.push_str(trimmed);
            }
        }

        if section_text.is_empty() {
            None
        } else {
            Some(section_text)
        }
    }

    /// Extracts the first paragraph from text.
    fn extract_first_paragraph(text: &str) -> String {
        let mut result = String::new();
        let mut prev_empty = false;

        for line in text.lines() {
            let trimmed = line.trim();

            if trimmed.is_empty() {
                if !result.is_empty() {
                    prev_empty = true;
                }
                continue;
            }

            // Stop at second paragraph
            if prev_empty && !result.is_empty() {
                break;
            }

            if !result.is_empty() {
                result.push(' ');
            }
            result.push_str(trimmed);

            // Stop if we have enough text
            if result.len() > 400 {
                break;
            }
        }

        result
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

    #[test]
    fn test_parse_filename() {
        let (name, section) = ManpageScanner::parse_filename(Path::new("ls.1")).unwrap();
        assert_eq!(name, "ls");
        assert_eq!(section, "1");

        let (name, section) = ManpageScanner::parse_filename(Path::new("mount.8.gz")).unwrap();
        assert_eq!(name, "mount");
        assert_eq!(section, "8");

        let (name, section) = ManpageScanner::parse_filename(Path::new("git-commit.1")).unwrap();
        assert_eq!(name, "git-commit");
        assert_eq!(section, "1");
    }

    #[test]
    fn test_extract_section() {
        let content = "NAME\n       ls - list directory contents\n\nDESCRIPTION\n       List information about the FILEs.";

        let name = ManpageScanner::extract_section(content, "NAME");
        assert!(name.is_some());
        assert!(name.unwrap().contains("list directory"));

        let desc = ManpageScanner::extract_section(content, "DESCRIPTION");
        assert!(desc.is_some());
        assert!(desc.unwrap().contains("FILEs"));
    }

    #[test]
    fn test_parse_manpage_content() {
        let content = "NAME\n       ls - list directory contents\n\nDESCRIPTION\n       List information about the FILEs.";

        let result = ManpageScanner::parse_manpage_content(content, "ls");
        assert!(result.contains("ls"));
        assert!(result.contains("list"));
    }

    #[test]
    fn test_extract_first_paragraph() {
        let text = "First paragraph line one. Line two.\n\nSecond paragraph.";
        let para = ManpageScanner::extract_first_paragraph(text);
        assert!(para.contains("First paragraph"));
        assert!(!para.contains("Second"));
    }
}
