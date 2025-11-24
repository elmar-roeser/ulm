//! Setup and initialization orchestration.
//!
//! This module handles the `ulm setup` command, including:
//! - Ollama detection and health checking
//! - Model verification and pulling
//! - Manpage scanning and indexing

pub mod config;
pub mod index;
pub mod install;
pub mod metadata;
pub mod models;
pub mod ollama;

pub use config::{
    get_config_path, load_config, save_config, Config, IndexConfig, ModelsConfig, OllamaConfig,
};
pub use index::{EmbeddingGenerator, ManpageContent, ManpageEntry, ManpageScanner};
pub use install::{
    detect_system, display_status, install_docker, install_native, start_ollama, wait_for_ollama,
    InstallResult, OllamaStatus, SystemCapabilities,
};
pub use metadata::IndexMetadata;
pub use models::{
    display_embedding_model_selection, display_model_selection, display_preset_selection,
    get_available_embedding_models, get_available_models, get_system_ram_gb,
    pull_model_with_progress, EmbeddingModel, ModelPreset, PresetSelection, PullProgress,
    RecommendedModel, MODEL_PRESETS,
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
#[allow(clippy::too_many_lines)]
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

    // Detect system RAM
    let system_ram = get_system_ram_gb();
    println!("System RAM: {system_ram:.1} GB");

    // Ask for preset or custom selection
    let preset_selection =
        display_preset_selection(system_ram).context("Failed to display preset selection")?;

    let (embedding_model_name, llm_model_name) = match preset_selection {
        PresetSelection::Preset(idx) => {
            let preset = &MODEL_PRESETS[idx];
            info!(preset = preset.name, "User selected preset");
            println!("\nUsing '{}' preset:", preset.name);
            println!("  Embedding: {}", preset.embedding_model);
            println!("  LLM: {}\n", preset.llm_model);
            (
                preset.embedding_model.to_string(),
                preset.llm_model.to_string(),
            )
        }
        PresetSelection::Custom => {
            // Step 2a: Embedding model selection
            println!("\nFetching embedding models...");
            let embedding_models = get_available_embedding_models(&client)
                .await
                .context("Failed to fetch embedding models")?;

            let embedding_idx = display_embedding_model_selection(&embedding_models)
                .context("Failed to display embedding model selection")?;

            let selected_embedding = &embedding_models[embedding_idx];
            info!(model = %selected_embedding.name, "User selected embedding model");

            // Step 2b: LLM model selection
            println!("\nFetching LLM models...");
            let llm_models = get_available_models(&client)
                .await
                .context("Failed to fetch LLM models")?;

            let llm_idx = display_model_selection(&llm_models, system_ram)
                .context("Failed to display LLM model selection")?;

            let selected_llm = &llm_models[llm_idx];
            info!(model = %selected_llm.name, "User selected LLM model");

            (selected_embedding.name.clone(), selected_llm.name.clone())
        }
    };

    // Check and pull embedding model
    let installed_models = client
        .list_models()
        .await
        .context("Failed to get installed models")?;
    let installed_names: Vec<&str> = installed_models.iter().map(|m| m.name.as_str()).collect();

    let embedding_installed = installed_names.iter().any(|name| {
        *name == embedding_model_name || name.starts_with(&format!("{embedding_model_name}:"))
    });

    if embedding_installed {
        println!("✓ Embedding model '{embedding_model_name}' already installed");
    } else {
        println!("Downloading embedding model {embedding_model_name}...");
        pull_model_with_progress(&client, &embedding_model_name)
            .await
            .context("Failed to pull embedding model")?;
        println!("✓ Embedding model '{embedding_model_name}' downloaded");
    }

    // Check and pull LLM model
    let llm_installed = installed_names
        .iter()
        .any(|name| *name == llm_model_name || name.starts_with(&format!("{llm_model_name}:")));

    if llm_installed {
        println!("✓ LLM model '{llm_model_name}' already installed\n");
    } else {
        println!("Downloading LLM model {llm_model_name}...");
        pull_model_with_progress(&client, &llm_model_name)
            .await
            .context("Failed to pull LLM model")?;
        println!("✓ LLM model '{llm_model_name}' downloaded\n");
    }

    // Save configuration
    let config = Config {
        models: ModelsConfig {
            embedding_model: embedding_model_name,
            llm_model: llm_model_name,
        },
        ollama: OllamaConfig {
            url: "http://localhost:11434".to_string(),
        },
        index: IndexConfig {
            embedding_dimension: None, // Will be set after indexing
            last_embedding_model: None,
        },
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

/// Removes all ulm data (database and config).
///
/// # Errors
///
/// Returns an error if deletion fails.
pub fn run_clean() -> Result<()> {
    println!("ulm clean - Removing all data\n");

    let mut removed = false;

    // Remove database
    let db_path = db::get_database_path()?;
    if db_path.exists() {
        std::fs::remove_dir_all(&db_path)
            .with_context(|| format!("Failed to remove database: {}", db_path.display()))?;
        println!("✓ Removed database: {}", db_path.display());
        removed = true;
    }

    // Remove metadata
    let metadata_path = db_path
        .parent()
        .map(|p| p.join("index_metadata.json"))
        .unwrap_or_default();
    if metadata_path.exists() {
        std::fs::remove_file(&metadata_path)
            .with_context(|| format!("Failed to remove metadata: {}", metadata_path.display()))?;
        println!("✓ Removed metadata: {}", metadata_path.display());
        removed = true;
    }

    // Remove config
    let config_path = get_config_path()?;
    if config_path.exists() {
        std::fs::remove_file(&config_path)
            .with_context(|| format!("Failed to remove config: {}", config_path.display()))?;
        println!("✓ Removed config: {}", config_path.display());
        removed = true;
    }

    if removed {
        println!("\n✓ Clean complete!");
    } else {
        println!("Nothing to clean - no ulm data found.");
    }

    Ok(())
}

/// Runs the indexing steps (shared between setup and update).
///
/// Uses pipelined processing: extraction and embedding run in parallel.
/// Supports incremental updates by tracking file hashes.
///
/// # Errors
///
/// Returns an error if any step fails.
async fn run_indexing() -> Result<usize> {
    // Step 3: Scan manpage directories
    println!("Scanning manpage directories...");
    let scanner = ManpageScanner::new();
    let all_paths = scanner
        .scan_directories()
        .context("Failed to scan manpage directories")?;
    let total_paths = all_paths.len();
    println!("✓ Found {total_paths} manpages\n");

    if total_paths == 0 {
        println!("No manpages found. Check your MANPATH environment variable.");
        return Ok(0);
    }

    // Load metadata and filter to changed files
    let mut metadata = IndexMetadata::load().unwrap_or_default();
    metadata.remove_deleted();

    let (paths_to_process, unchanged) = metadata.filter_changed(all_paths.clone());
    let to_process_count = paths_to_process.len();

    if to_process_count == 0 {
        println!("✓ All {unchanged} manpages unchanged - nothing to update\n");
        return Ok(unchanged);
    }

    if unchanged > 0 {
        println!("  {unchanged} unchanged, {to_process_count} to process\n");
    }

    // Steps 4-5: Extract and embed in pipeline (only changed files)
    println!("Extracting and generating embeddings (pipelined)...");

    let generator = EmbeddingGenerator::new().context("Failed to create embedding generator")?;
    let entries = generator
        .generate_embeddings_pipelined(paths_to_process.clone())
        .await
        .context("Failed to generate embeddings")?;

    let entry_count = entries.len();
    println!("✓ Generated {entry_count} embeddings\n");

    // Update config with embedding dimension
    if let Some(first_entry) = entries.first() {
        #[allow(clippy::cast_possible_truncation)]
        let dimension = first_entry.vector.len() as u32;
        let mut config = load_config().context("Failed to load config")?;
        config.update_index_metadata(dimension);
        save_config(&config).context("Failed to save config with index metadata")?;
        info!(dimension = dimension, model = %config.embedding_model(), "Saved index metadata to config");
    }

    // Update metadata with processed files
    metadata.update_hashes(&paths_to_process);
    metadata.save().context("Failed to save metadata")?;

    // Step 6: Store in LanceDB
    println!("Storing in database...");
    db::create_index(entries)
        .await
        .context("Failed to create database index")?;

    info!(count = entry_count, "Indexing complete");

    Ok(entry_count)
}
