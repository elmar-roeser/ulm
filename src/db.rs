//! `SQLite`-vec operations for vector storage and search.
//!
//! This module handles all interactions with the `SQLite` database
//! using the sqlite-vec extension for vector similarity search.

#![allow(unsafe_code)] // Required for loading SQLite extensions

use std::fs;
use std::path::PathBuf;

use anyhow::{Context, Result};
use rusqlite::ffi::sqlite3_auto_extension;
use rusqlite::Connection;
use tracing::{debug, info};
use zerocopy::AsBytes;

use crate::setup::ManpageEntry;

/// Name of the database file.
const DB_FILENAME: &str = "index.db";

/// Gets the path to the `SQLite` database.
///
/// Uses XDG Base Directory specification:
/// - Linux: ~/.local/share/ulm/index.db
/// - macOS: ~/Library/Application Support/ulm/index.db
///
/// # Errors
///
/// Returns an error if the data directory cannot be determined.
pub fn get_database_path() -> Result<PathBuf> {
    let dirs = directories::ProjectDirs::from("", "", "ulm")
        .context("Could not determine data directory")?;

    let data_dir = dirs.data_dir();

    // Create parent directories if needed
    fs::create_dir_all(data_dir)
        .with_context(|| format!("Failed to create data directory: {}", data_dir.display()))?;

    Ok(data_dir.join(DB_FILENAME))
}

/// Initialize sqlite-vec as an auto-extension (called once at startup).
fn init_sqlite_vec() {
    use std::sync::Once;
    static INIT: Once = Once::new();
    INIT.call_once(|| {
        // SAFETY: This registers sqlite-vec as an auto-extension that loads
        // automatically for all new connections. The transmute converts the
        // init function pointer to the expected callback signature.
        #[allow(clippy::missing_transmute_annotations)]
        unsafe {
            sqlite3_auto_extension(Some(std::mem::transmute(
                sqlite_vec::sqlite3_vec_init as *const (),
            )));
        }
    });
}

/// Opens a connection to the database with sqlite-vec loaded.
fn open_connection(path: &PathBuf) -> Result<Connection> {
    // Initialize sqlite-vec auto-extension before opening connection
    init_sqlite_vec();

    let conn = Connection::open(path)
        .with_context(|| format!("Failed to open database: {}", path.display()))?;

    Ok(conn)
}

/// Creates or overwrites the vector index with the given entries.
///
/// # Errors
///
/// Returns an error if database operations fail.
#[allow(clippy::unused_async)] // Keep async for API compatibility
pub async fn create_index(entries: Vec<ManpageEntry>) -> Result<()> {
    let db_path = get_database_path()?;

    info!(path = %db_path.display(), entries = entries.len(), "Creating vector index");

    let conn = open_connection(&db_path)?;

    // Get vector dimension from first entry
    let vector_dim = entries.first().map_or(768, |e| e.vector.len());

    // Drop existing tables
    conn.execute("DROP TABLE IF EXISTS manpages_vec", [])
        .context("Failed to drop vector table")?;
    conn.execute("DROP TABLE IF EXISTS manpages", [])
        .context("Failed to drop manpages table")?;

    // Create metadata table
    conn.execute(
        "CREATE TABLE manpages (
            id INTEGER PRIMARY KEY,
            tool_name TEXT NOT NULL,
            section TEXT NOT NULL,
            description TEXT NOT NULL
        )",
        [],
    )
    .context("Failed to create manpages table")?;

    // Create virtual table for vectors
    conn.execute(
        &format!(
            "CREATE VIRTUAL TABLE manpages_vec USING vec0(
                id INTEGER PRIMARY KEY,
                embedding FLOAT[{vector_dim}]
            )"
        ),
        [],
    )
    .context("Failed to create vector table")?;

    // Insert entries
    let mut stmt = conn
        .prepare("INSERT INTO manpages (tool_name, section, description) VALUES (?1, ?2, ?3)")
        .context("Failed to prepare insert statement")?;

    let mut vec_stmt = conn
        .prepare("INSERT INTO manpages_vec (id, embedding) VALUES (?1, ?2)")
        .context("Failed to prepare vector insert statement")?;

    for entry in &entries {
        // Insert metadata
        stmt.execute(rusqlite::params![
            entry.tool_name,
            entry.section,
            entry.description
        ])
        .context("Failed to insert manpage")?;

        let id = conn.last_insert_rowid();

        // Insert vector as blob
        let vector_blob = entry.vector.as_bytes();
        vec_stmt
            .execute(rusqlite::params![id, vector_blob])
            .context("Failed to insert vector")?;
    }

    info!(
        "Created index with {} entries (dimension: {})",
        entries.len(),
        vector_dim
    );

    Ok(())
}

/// Checks if the vector index exists.
///
/// # Errors
///
/// Returns an error if database operations fail.
#[allow(clippy::unused_async)] // Keep async for API compatibility
pub async fn index_exists() -> Result<bool> {
    let db_path = get_database_path()?;

    // Check if database file exists
    if !db_path.exists() {
        return Ok(false);
    }

    let conn = open_connection(&db_path)?;

    // Check for manpages table
    let count: i64 = conn
        .query_row(
            "SELECT COUNT(*) FROM sqlite_master WHERE type='table' AND name='manpages'",
            [],
            |row| row.get(0),
        )
        .context("Failed to check table existence")?;

    Ok(count > 0)
}

/// Search result from vector similarity search.
#[derive(Debug, Clone)]
pub struct SearchResult {
    /// Tool name.
    pub tool_name: String,
    /// Man section.
    pub section: String,
    /// Description text.
    pub description: String,
    /// Similarity score (distance - lower is better).
    pub score: f32,
}

/// Performs vector similarity search on the index.
///
/// # Errors
///
/// Returns an error if database operations fail or index doesn't exist.
#[allow(clippy::unused_async)] // Keep async for API compatibility
pub async fn search(query_vector: &[f32], limit: usize) -> Result<Vec<SearchResult>> {
    let db_path = get_database_path()?;

    let conn = open_connection(&db_path)?;

    // Convert query vector to blob
    let query_blob = query_vector.as_bytes();

    // Perform vector search using sqlite-vec
    // The vec0 KNN query requires 'k = ?' constraint instead of LIMIT
    let mut stmt = conn
        .prepare(
            "SELECT
                m.tool_name,
                m.section,
                m.description,
                v.distance
            FROM manpages_vec v
            JOIN manpages m ON m.id = v.id
            WHERE v.embedding MATCH ?1 AND k = ?2
            ORDER BY v.distance",
        )
        .context("Failed to prepare search query")?;

    let results = stmt
        .query_map(rusqlite::params![query_blob, limit], |row| {
            Ok(SearchResult {
                tool_name: row.get(0)?,
                section: row.get(1)?,
                description: row.get(2)?,
                score: row.get(3)?,
            })
        })
        .context("Failed to execute search")?;

    let mut search_results = Vec::new();
    for result in results {
        search_results.push(result.context("Failed to read search result")?);
    }

    debug!(count = search_results.len(), "Vector search completed");

    Ok(search_results)
}

/// Gets the total number of entries in the index.
///
/// # Errors
///
/// Returns an error if database operations fail.
#[allow(clippy::unused_async)] // Keep async for API compatibility
pub async fn count_entries() -> Result<usize> {
    let db_path = get_database_path()?;

    let conn = open_connection(&db_path)?;

    let count: i64 = conn
        .query_row("SELECT COUNT(*) FROM manpages", [], |row| row.get(0))
        .context("Failed to count entries")?;

    #[allow(clippy::cast_sign_loss, clippy::cast_possible_truncation)]
    Ok(count as usize)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[allow(dead_code)]
    fn create_test_entries(count: usize) -> Vec<ManpageEntry> {
        (0..count)
            .map(|i| ManpageEntry {
                tool_name: format!("tool{i}"),
                section: "1".to_string(),
                description: format!("Description for tool {i}"),
                #[allow(clippy::cast_precision_loss)]
                vector: vec![0.1 * i as f32; 8], // Small vectors for testing
            })
            .collect()
    }

    #[test]
    fn test_get_database_path() {
        let path = get_database_path().unwrap();
        assert!(path.to_string_lossy().contains("ulm"));
        assert!(path.to_string_lossy().ends_with("index.db"));
    }
}
