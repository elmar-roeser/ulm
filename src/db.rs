//! `LanceDB` operations for vector storage and search.
//!
//! This module handles all interactions with the embedded `LanceDB`
//! database for storing and searching manpage embeddings.

use std::fs;
use std::path::PathBuf;
use std::sync::Arc;

use anyhow::{Context, Result};
use arrow_array::types::Float32Type;
use arrow_array::{
    Array, FixedSizeListArray, Float32Array, RecordBatch, RecordBatchIterator, StringArray,
};
use futures::TryStreamExt;
use lancedb::connect;
use lancedb::query::{ExecutableQuery, QueryBase};
use tracing::{debug, info};

use crate::setup::ManpageEntry;

/// Name of the manpages table in the database.
const TABLE_NAME: &str = "manpages";

/// Gets the path to the `LanceDB` database.
///
/// Uses XDG Base Directory specification:
/// - Linux: ~/.local/share/ulm/index.lance
/// - macOS: ~/Library/Application Support/ulm/index.lance
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

    Ok(data_dir.join("index.lance"))
}

/// Creates or overwrites the vector index with the given entries.
///
/// # Errors
///
/// Returns an error if database operations fail.
pub async fn create_index(entries: Vec<ManpageEntry>) -> Result<()> {
    let db_path = get_database_path()?;
    let db_uri = db_path.to_string_lossy();

    info!(path = %db_uri, entries = entries.len(), "Creating vector index");

    // Connect to database
    let db = connect(&db_uri)
        .execute()
        .await
        .with_context(|| format!("Failed to connect to database: {db_uri}"))?;

    // Check if table exists and drop it (overwrite mode)
    let existing_tables = db
        .table_names()
        .execute()
        .await
        .context("Failed to list tables")?;

    if existing_tables.contains(&TABLE_NAME.to_string()) {
        debug!("Dropping existing table '{TABLE_NAME}'");
        db.drop_table(TABLE_NAME, &[])
            .await
            .context("Failed to drop existing table")?;
    }

    // Create record batch from entries
    let batch = create_record_batch(&entries)?;

    // Create table with data
    let batches = RecordBatchIterator::new(vec![Ok(batch.clone())], batch.schema());

    db.create_table(TABLE_NAME, Box::new(batches))
        .execute()
        .await
        .context("Failed to create table")?;

    info!(
        "Created table '{}' with {} entries",
        TABLE_NAME,
        entries.len()
    );

    Ok(())
}

/// Creates an Arrow `RecordBatch` from manpage entries.
fn create_record_batch(entries: &[ManpageEntry]) -> Result<RecordBatch> {
    // Extract data into column vectors
    let tool_names: Vec<&str> = entries.iter().map(|e| e.tool_name.as_str()).collect();
    let sections: Vec<&str> = entries.iter().map(|e| e.section.as_str()).collect();
    let descriptions: Vec<&str> = entries.iter().map(|e| e.description.as_str()).collect();

    // Get vector dimension from first entry (assume all same size)
    // Vector dimensions are small (768-4096), so truncation is not a concern
    #[allow(clippy::cast_possible_truncation, clippy::cast_possible_wrap)]
    let vector_dim = entries.first().map_or(768, |e| e.vector.len()) as i32;

    // Create Arrow arrays
    let tool_name_array = Arc::new(StringArray::from(tool_names));
    let section_array = Arc::new(StringArray::from(sections));
    let description_array = Arc::new(StringArray::from(descriptions));

    // Create fixed-size list array for vectors
    let vector_array = Arc::new(
        FixedSizeListArray::from_iter_primitive::<Float32Type, _, _>(
            entries
                .iter()
                .map(|e| Some(e.vector.iter().copied().map(Some).collect::<Vec<_>>())),
            vector_dim,
        ),
    );

    // Create record batch with schema
    let batch = RecordBatch::try_from_iter(vec![
        ("tool_name", tool_name_array as Arc<dyn Array>),
        ("section", section_array as Arc<dyn Array>),
        ("description", description_array as Arc<dyn Array>),
        ("vector", vector_array as Arc<dyn Array>),
    ])
    .context("Failed to create record batch")?;

    debug!(
        rows = batch.num_rows(),
        columns = batch.num_columns(),
        vector_dim = vector_dim,
        "Created record batch"
    );

    Ok(batch)
}

/// Checks if the vector index exists.
///
/// # Errors
///
/// Returns an error if database operations fail.
pub async fn index_exists() -> Result<bool> {
    let db_path = get_database_path()?;

    // Check if database directory exists
    if !db_path.exists() {
        return Ok(false);
    }

    let db_uri = db_path.to_string_lossy();

    // Connect and check for table
    let db = connect(&db_uri)
        .execute()
        .await
        .with_context(|| format!("Failed to connect to database: {db_uri}"))?;

    let tables = db
        .table_names()
        .execute()
        .await
        .context("Failed to list tables")?;

    Ok(tables.contains(&TABLE_NAME.to_string()))
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
pub async fn search(query_vector: &[f32], limit: usize) -> Result<Vec<SearchResult>> {
    let db_path = get_database_path()?;
    let db_uri = db_path.to_string_lossy();

    let db = connect(&db_uri)
        .execute()
        .await
        .with_context(|| format!("Failed to connect to database: {db_uri}"))?;

    let table = db
        .open_table(TABLE_NAME)
        .execute()
        .await
        .context("Failed to open manpages table")?;

    // Perform vector search
    let mut results = table
        .vector_search(query_vector)
        .context("Failed to create vector search")?
        .limit(limit)
        .execute()
        .await
        .context("Failed to execute vector search")?;

    // Convert results to our type
    let mut search_results = Vec::new();

    while let Some(batch) = results
        .try_next()
        .await
        .context("Failed to read result batch")?
    {
        let tool_names = batch
            .column_by_name("tool_name")
            .context("Missing tool_name column")?
            .as_any()
            .downcast_ref::<StringArray>()
            .context("Invalid tool_name type")?;

        let sections = batch
            .column_by_name("section")
            .context("Missing section column")?
            .as_any()
            .downcast_ref::<StringArray>()
            .context("Invalid section type")?;

        let descriptions = batch
            .column_by_name("description")
            .context("Missing description column")?
            .as_any()
            .downcast_ref::<StringArray>()
            .context("Invalid description type")?;

        let scores = batch
            .column_by_name("_distance")
            .context("Missing _distance column")?
            .as_any()
            .downcast_ref::<Float32Array>()
            .context("Invalid _distance type")?;

        for i in 0..batch.num_rows() {
            search_results.push(SearchResult {
                tool_name: tool_names.value(i).to_string(),
                section: sections.value(i).to_string(),
                description: descriptions.value(i).to_string(),
                score: scores.value(i),
            });
        }
    }

    debug!(count = search_results.len(), "Vector search completed");

    Ok(search_results)
}

/// Gets the total number of entries in the index.
///
/// # Errors
///
/// Returns an error if database operations fail.
pub async fn count_entries() -> Result<usize> {
    let db_path = get_database_path()?;
    let db_uri = db_path.to_string_lossy();

    let db = connect(&db_uri)
        .execute()
        .await
        .with_context(|| format!("Failed to connect to database: {db_uri}"))?;

    let table = db
        .open_table(TABLE_NAME)
        .execute()
        .await
        .context("Failed to open manpages table")?;

    let count = table
        .count_rows(None)
        .await
        .context("Failed to count rows")?;

    Ok(count)
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    fn create_test_entries(count: usize) -> Vec<ManpageEntry> {
        (0..count)
            .map(|i| ManpageEntry {
                tool_name: format!("tool{i}"),
                section: "1".to_string(),
                description: format!("Description for tool {i}"),
                vector: vec![0.1 * i as f32; 8], // Small vectors for testing
            })
            .collect()
    }

    #[test]
    fn test_get_database_path() {
        let path = get_database_path().unwrap();
        assert!(path.to_string_lossy().contains("ulm"));
        assert!(path.to_string_lossy().ends_with("index.lance"));
    }

    #[test]
    fn test_create_record_batch() {
        let entries = create_test_entries(3);
        let batch = create_record_batch(&entries).unwrap();

        assert_eq!(batch.num_rows(), 3);
        assert_eq!(batch.num_columns(), 4);
    }

    #[test]
    fn test_create_record_batch_empty() {
        let entries: Vec<ManpageEntry> = vec![];
        let batch = create_record_batch(&entries).unwrap();

        assert_eq!(batch.num_rows(), 0);
    }

    #[tokio::test]
    async fn test_create_and_check_index() {
        // Use temp directory for test
        let temp_dir = TempDir::new().unwrap();
        let db_path = temp_dir.path().join("test.lance");

        // Create entries
        let entries = create_test_entries(5);

        // Manually create index at test path
        let db_uri = db_path.to_string_lossy();
        let db = connect(&db_uri).execute().await.unwrap();

        let batch = create_record_batch(&entries).unwrap();
        let batches = RecordBatchIterator::new(vec![Ok(batch.clone())], batch.schema());

        db.create_table(TABLE_NAME, Box::new(batches))
            .execute()
            .await
            .unwrap();

        // Verify table exists
        let tables = db.table_names().execute().await.unwrap();
        assert!(tables.contains(&TABLE_NAME.to_string()));
    }

    #[tokio::test]
    async fn test_overwrite_existing_index() {
        let temp_dir = TempDir::new().unwrap();
        let db_path = temp_dir.path().join("test.lance");
        let db_uri = db_path.to_string_lossy();

        // Create first index
        let entries1 = create_test_entries(3);
        let db = connect(&db_uri).execute().await.unwrap();

        let batch1 = create_record_batch(&entries1).unwrap();
        let batches1 = RecordBatchIterator::new(vec![Ok(batch1.clone())], batch1.schema());
        db.create_table(TABLE_NAME, Box::new(batches1))
            .execute()
            .await
            .unwrap();

        // Drop and recreate with different count
        db.drop_table(TABLE_NAME, &[]).await.unwrap();

        let entries2 = create_test_entries(7);
        let batch2 = create_record_batch(&entries2).unwrap();
        let batches2 = RecordBatchIterator::new(vec![Ok(batch2.clone())], batch2.schema());
        db.create_table(TABLE_NAME, Box::new(batches2))
            .execute()
            .await
            .unwrap();

        // Verify new count
        let table = db.open_table(TABLE_NAME).execute().await.unwrap();
        let count = table.count_rows(None).await.unwrap();
        assert_eq!(count, 7);
    }
}
