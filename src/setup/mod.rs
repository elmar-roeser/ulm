//! Setup and initialization orchestration.
//!
//! This module handles the `ulm setup` command, including:
//! - Ollama detection and health checking
//! - Model verification and pulling
//! - Manpage scanning and indexing

pub mod config;
pub mod index;
pub mod install;
pub mod models;
pub mod ollama;

pub use config::{get_config_path, load_config, save_config, Config};
pub use index::{EmbeddingGenerator, ManpageContent, ManpageEntry, ManpageScanner};
pub use install::{
    detect_system, display_status, install_docker, install_native, start_ollama, wait_for_ollama,
    InstallResult, OllamaStatus, SystemCapabilities,
};
pub use models::{
    display_model_selection, get_available_models, get_system_ram_gb, pull_model_with_progress,
    PullProgress, RecommendedModel,
};
pub use ollama::OllamaChecker;

use anyhow::{Context, Result};
use tracing::info;

use crate::db;
use crate::llm::OllamaClient;

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

    // Step 1: Detect system and ensure Ollama is running
    println!("Detecting system...");
    let caps = detect_system().await?;
    display_status(&caps);

    match caps.ollama_status {
        OllamaStatus::Running => {
            println!("✓ Ollama is running\n");
        }
        OllamaStatus::Installed => {
            println!("Ollama is installed but not running. Starting...");
            start_ollama().await?;
            wait_for_ollama(30).await?;
            println!("✓ Ollama started\n");
        }
        OllamaStatus::NotInstalled => {
            println!("\nOllama is not installed. How would you like to install it?\n");
            println!("  1. Native installation (recommended)");
            println!("  2. Docker container");
            println!("  3. Cancel setup\n");
            print!("Select option [1-3]: ");
            std::io::Write::flush(&mut std::io::stdout())?;

            let mut input = String::new();
            std::io::stdin().read_line(&mut input)?;

            match input.trim() {
                "1" => {
                    let result = install_native(&caps.os).await?;
                    if !result.success {
                        anyhow::bail!("Installation failed: {}", result.message);
                    }
                    println!("{}", result.message);
                    start_ollama().await?;
                    wait_for_ollama(30).await?;
                    println!("✓ Ollama installed and started\n");
                }
                "2" => {
                    let result = install_docker("run").await?;
                    if !result.success {
                        anyhow::bail!("Installation failed: {}", result.message);
                    }
                    println!("{}", result.message);
                    wait_for_ollama(30).await?;
                    println!("✓ Ollama Docker container started\n");
                }
                _ => {
                    anyhow::bail!("Setup cancelled");
                }
            }
        }
    }

    // Step 2: Model selection and setup
    let client = OllamaClient::new().context("Failed to create Ollama client")?;

    // Get available models with installed status
    println!("Fetching available models...");
    let models = get_available_models(&client)
        .await
        .context("Failed to fetch available models")?;

    // Detect system RAM and display selection
    let system_ram = get_system_ram_gb();
    println!("System RAM: {system_ram:.1} GB\n");

    let selected_idx = display_model_selection(&models, system_ram)
        .context("Failed to display model selection")?;

    let selected_model = &models[selected_idx];
    info!(model = %selected_model.name, "User selected model");

    // Pull model if not installed
    if selected_model.installed {
        println!("\n✓ Model '{}' already installed\n", selected_model.name);
    } else {
        println!("\nDownloading {}...", selected_model.name);
        pull_model_with_progress(&client, &selected_model.name)
            .await
            .context("Failed to pull model")?;
        println!("✓ Model '{}' downloaded\n", selected_model.name);
    }

    // Save configuration
    let config = Config {
        model_name: selected_model.name.clone(),
        ollama_url: "http://localhost:11434".to_string(),
    };
    save_config(&config).context("Failed to save configuration")?;
    println!(
        "✓ Configuration saved to {}\n",
        get_config_path()?.display()
    );

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
