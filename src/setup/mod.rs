//! Setup and initialization orchestration.
//!
//! This module handles the `ulm setup` command, including:
//! - Ollama detection and health checking
//! - Model verification and pulling
//! - Manpage scanning and indexing

pub mod index;
pub mod install;
pub mod ollama;

pub use index::{EmbeddingGenerator, ManpageContent, ManpageEntry, ManpageScanner};
pub use install::{detect_system, display_status, OllamaStatus, SystemCapabilities};
pub use ollama::OllamaChecker;

use anyhow::{Context, Result};
use tracing::info;

use crate::db;

/// Runs the complete setup process.
///
/// Steps:
/// 1. Check Ollama connection
/// 2. Verify/pull model
/// 3. Scan manpage directories
/// 4. Extract descriptions
/// 5. Generate embeddings
/// 6. Store in `LanceDB`
///
/// # Errors
///
/// Returns an error if any step fails.
pub async fn run_setup() -> Result<()> {
    println!("ulm setup - Initializing manpage index\n");

    // Step 1: Check Ollama connection
    println!("Checking Ollama connection...");
    let checker = OllamaChecker::new().context("Failed to create Ollama checker")?;
    checker
        .check_connection()
        .await
        .context("Failed to connect to Ollama")?;
    println!("✓ Ollama detected at localhost:11434\n");

    // Step 2: Verify/pull model
    println!("Checking for LLM model...");
    let model = checker
        .check_model()
        .await
        .context("Failed to check model")?;
    println!("✓ Model '{model}' available\n");

    // Steps 3-6: Run indexing
    let count = run_indexing().await?;

    println!("\n✓ Setup complete! Indexed {count} manpages");
    println!(
        "  Database location: {}",
        db::get_database_path()?.display()
    );

    Ok(())
}

/// Runs the update process (re-index without Ollama checks).
///
/// Steps:
/// 1. Scan manpage directories
/// 2. Extract descriptions
/// 3. Generate embeddings
/// 4. Store in `LanceDB` (overwrites existing)
///
/// # Errors
///
/// Returns an error if any step fails.
pub async fn run_update() -> Result<()> {
    println!("ulm update - Refreshing manpage index\n");

    // Run indexing steps
    let count = run_indexing().await?;

    println!("\n✓ Update complete! Indexed {count} manpages");
    println!(
        "  Database location: {}",
        db::get_database_path()?.display()
    );

    Ok(())
}

/// Runs the indexing steps (shared between setup and update).
///
/// # Errors
///
/// Returns an error if any step fails.
async fn run_indexing() -> Result<usize> {
    // Step 3: Scan manpage directories
    println!("Scanning manpage directories...");
    let scanner = ManpageScanner::new();
    let paths = scanner
        .scan_directories()
        .context("Failed to scan manpage directories")?;
    let total_paths = paths.len();
    println!("✓ Found {total_paths} manpages\n");

    if total_paths == 0 {
        println!("No manpages found. Check your MANPATH environment variable.");
        return Ok(0);
    }

    // Step 4: Extract descriptions
    println!("Extracting descriptions...");
    let mut contents = Vec::with_capacity(total_paths);
    let mut errors = 0;

    for (i, path) in paths.iter().enumerate() {
        // Progress every 100 items
        if i % 100 == 0 || i == total_paths - 1 {
            print!("\r  Extracting... {}/{}", i + 1, total_paths);
            std::io::Write::flush(&mut std::io::stdout()).ok();
        }

        match ManpageScanner::extract_content(path) {
            Ok(content) => contents.push(content),
            Err(e) => {
                errors += 1;
                tracing::debug!(path = ?path, error = %e, "Failed to extract content");
            }
        }
    }
    println!();

    if errors > 0 {
        println!("  (Skipped {errors} malformed manpages)");
    }
    println!("✓ Extracted {} descriptions\n", contents.len());

    if contents.is_empty() {
        println!("No descriptions extracted. Cannot proceed with indexing.");
        return Ok(0);
    }

    // Step 5: Generate embeddings
    println!("Generating embeddings...");
    let generator = EmbeddingGenerator::new().context("Failed to create embedding generator")?;
    let entries = generator
        .generate_embeddings(contents)
        .await
        .context("Failed to generate embeddings")?;

    let entry_count = entries.len();
    println!("✓ Generated {entry_count} embeddings\n");

    // Step 6: Store in LanceDB
    println!("Storing in database...");
    db::create_index(entries)
        .await
        .context("Failed to create database index")?;

    info!(count = entry_count, "Indexing complete");

    Ok(entry_count)
}
