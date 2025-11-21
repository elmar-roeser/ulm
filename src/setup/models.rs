//! Model management and recommendation for Ollama setup.
//!
//! This module provides functionality to fetch available models from Ollama
//! and merge them with a curated list of recommended models.

use std::io::{self, Write};
use std::time::Duration;

use anyhow::{Context, Result};
use indicatif::{ProgressBar, ProgressStyle};
use serde::{Deserialize, Serialize};
use sysinfo::System;

use crate::llm::ollama::OllamaClient;

/// Information about a recommended Ollama model.
///
/// This struct contains metadata about models suitable for ulm,
/// including RAM requirements and quality/speed ratings.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct RecommendedModel {
    /// Model name (e.g., "llama3.2:3b").
    pub name: String,
    /// Estimated RAM requirement in GB.
    pub ram_gb: f32,
    /// Speed rating from 1-5 (5 = fastest).
    pub speed_rating: u8,
    /// Quality rating from 1-5 (5 = best).
    pub quality_rating: u8,
    /// Whether this model is installed in Ollama.
    pub installed: bool,
}

/// Curated list of recommended models with their characteristics.
///
/// These models are selected for good performance with ulm's use case
/// (command generation from manpages).
pub const RECOMMENDED_MODELS: &[RecommendedModel] = &[
    RecommendedModel {
        name: String::new(), // Will be set at runtime
        ram_gb: 4.0,
        speed_rating: 5,
        quality_rating: 3,
        installed: false,
    },
    RecommendedModel {
        name: String::new(),
        ram_gb: 6.0,
        speed_rating: 4,
        quality_rating: 4,
        installed: false,
    },
    RecommendedModel {
        name: String::new(),
        ram_gb: 8.0,
        speed_rating: 3,
        quality_rating: 5,
        installed: false,
    },
    RecommendedModel {
        name: String::new(),
        ram_gb: 3.0,
        speed_rating: 5,
        quality_rating: 2,
        installed: false,
    },
];

/// Returns the default list of recommended models with their names.
fn get_default_models() -> Vec<RecommendedModel> {
    vec![
        RecommendedModel {
            name: "llama3.2:3b".to_string(),
            ram_gb: 4.0,
            speed_rating: 5,
            quality_rating: 3,
            installed: false,
        },
        RecommendedModel {
            name: "mistral:7b".to_string(),
            ram_gb: 6.0,
            speed_rating: 4,
            quality_rating: 4,
            installed: false,
        },
        RecommendedModel {
            name: "llama3.1:8b".to_string(),
            ram_gb: 8.0,
            speed_rating: 3,
            quality_rating: 5,
            installed: false,
        },
        RecommendedModel {
            name: "phi3:mini".to_string(),
            ram_gb: 3.0,
            speed_rating: 5,
            quality_rating: 2,
            installed: false,
        },
    ]
}

/// Fetches available models from Ollama and merges with recommended list.
///
/// This function:
/// 1. Queries Ollama `/api/tags` for installed models
/// 2. Merges with the recommended models list
/// 3. Sets `installed: true` for models found in Ollama
///
/// # Arguments
///
/// * `client` - The Ollama client to use for API calls
///
/// # Returns
///
/// A vector of `RecommendedModel` with installation status updated.
///
/// # Errors
///
/// Returns an error if the Ollama API call fails.
pub async fn get_available_models(client: &OllamaClient) -> Result<Vec<RecommendedModel>> {
    // Get list of installed models from Ollama
    let installed_models = client
        .list_models()
        .await
        .context("Failed to fetch installed models from Ollama")?;

    // Extract installed model names for easy lookup
    let installed_names: Vec<String> = installed_models.iter().map(|m| m.name.clone()).collect();

    // Get default recommended models and update their installed status
    let mut models = get_default_models();

    for model in &mut models {
        // Check if model is installed (exact match or with :latest suffix)
        model.installed = installed_names.iter().any(|name| {
            name == &model.name
                || name
                    == &format!(
                        "{}:latest",
                        model.name.split(':').next().unwrap_or(&model.name)
                    )
                || model.name == format!("{}:latest", name.split(':').next().unwrap_or(name))
        });
    }

    Ok(models)
}

/// Gets the system's total RAM in gigabytes.
///
/// # Returns
///
/// The total system RAM in GB as a float.
#[must_use]
pub fn get_system_ram_gb() -> f32 {
    let sys = System::new_all();
    let total_memory_bytes = sys.total_memory();
    // Convert bytes to GB (1 GB = 1024^3 bytes)
    // Allow precision loss as we don't need exact byte-level accuracy for RAM
    #[allow(clippy::cast_precision_loss)]
    let ram_gb = total_memory_bytes as f32 / (1024.0 * 1024.0 * 1024.0);
    ram_gb
}

/// Displays the model selection table and returns the selected model index.
///
/// This function:
/// 1. Prints a formatted table of available models
/// 2. Highlights the recommended model based on system RAM
/// 3. Prompts the user to select a model
/// 4. Returns the selected index (0-based)
///
/// # Arguments
///
/// * `models` - The list of available models to display
/// * `system_ram_gb` - The system's total RAM in GB
///
/// # Returns
///
/// The index of the selected model (0-based).
///
/// # Errors
///
/// Returns an error if reading user input fails.
pub fn display_model_selection(models: &[RecommendedModel], system_ram_gb: f32) -> Result<usize> {
    // Determine recommended model based on RAM
    let recommended_idx = get_recommended_model_index(system_ram_gb);

    println!("\nAvailable Models:\n");
    println!(
        " #  {:<14} {:<7} {:<7} {:<7} Status",
        "Model", "RAM", "Speed", "Quality"
    );
    println!("{}", "-".repeat(60));

    for (i, model) in models.iter().enumerate() {
        let speed_stars = format_stars(model.speed_rating);
        let quality_stars = format_stars(model.quality_rating);

        let status = if model.installed && i == recommended_idx {
            "[Installed] [Recommended]"
        } else if model.installed {
            "[Installed]"
        } else if i == recommended_idx {
            "[Recommended]"
        } else {
            ""
        };

        println!(
            " {}  {:<14} {:<7} {:<7} {:<7} {}",
            i + 1,
            model.name,
            format!("{:.0} GB", model.ram_gb),
            speed_stars,
            quality_stars,
            status
        );
    }

    println!();
    println!(
        "Your system has {:.1} GB RAM. Recommended: {}",
        system_ram_gb,
        models.get(recommended_idx).map_or("unknown", |m| &m.name)
    );
    println!();

    // Read user selection
    loop {
        print!("Select model (1-{}): ", models.len());
        io::stdout().flush().context("Failed to flush stdout")?;

        let mut input = String::new();
        io::stdin()
            .read_line(&mut input)
            .context("Failed to read user input")?;

        match input.trim().parse::<usize>() {
            Ok(n) if n >= 1 && n <= models.len() => return Ok(n - 1),
            _ => {
                println!(
                    "Invalid selection. Please enter a number between 1 and {}.",
                    models.len()
                );
            }
        }
    }
}

/// Formats a rating (1-5) as star characters.
fn format_stars(rating: u8) -> String {
    let filled = rating.min(5) as usize;
    let empty = 5 - filled;
    format!("{}{}", "★".repeat(filled), "☆".repeat(empty))
}

/// Gets the recommended model index based on system RAM.
///
/// Recommendation logic:
/// - >= 8 GB: llama3.1:8b (index 2) - best quality
/// - >= 6 GB: mistral:7b (index 1) - balance
/// - >= 4 GB: llama3.2:3b (index 0) - good speed
/// - < 4 GB: phi3:mini (index 3) - minimal
fn get_recommended_model_index(ram_gb: f32) -> usize {
    if ram_gb >= 8.0 {
        2 // llama3.1:8b
    } else if ram_gb >= 6.0 {
        1 // mistral:7b
    } else if ram_gb >= 4.0 {
        0 // llama3.2:3b
    } else {
        3 // phi3:mini
    }
}

/// Progress update from Ollama pull API.
#[derive(Debug, Clone, Deserialize)]
pub struct PullProgress {
    /// Status message (e.g., "downloading", "verifying", "success").
    pub status: String,
    /// Digest of the current layer being downloaded.
    #[serde(default)]
    pub digest: Option<String>,
    /// Total size in bytes.
    #[serde(default)]
    pub total: Option<u64>,
    /// Completed size in bytes.
    #[serde(default)]
    pub completed: Option<u64>,
}

/// Request body for pulling a model.
#[derive(Debug, Clone, Serialize)]
struct PullRequest {
    /// Model name to pull.
    name: String,
    /// Whether to stream progress updates.
    stream: bool,
}

/// Pulls a model from Ollama with progress display.
///
/// This function:
/// 1. Sends a streaming pull request to Ollama
/// 2. Parses JSON lines for progress updates
/// 3. Displays a progress bar during download
///
/// # Arguments
///
/// * `client` - The Ollama client to use for API calls
/// * `model_name` - Name of the model to pull (e.g., "llama3.2:3b")
///
/// # Errors
///
/// Returns an error if the pull fails or network errors occur.
pub async fn pull_model_with_progress(client: &OllamaClient, model_name: &str) -> Result<()> {
    let url = format!("{}/api/pull", client.base_url());

    let request = PullRequest {
        name: model_name.to_string(),
        stream: true,
    };

    println!("Pulling model '{model_name}'...");

    // Create HTTP client with long timeout for large models
    let http_client = reqwest::Client::builder()
        .timeout(Duration::from_secs(1800)) // 30 minutes
        .build()
        .context("Failed to create HTTP client")?;

    let response = http_client
        .post(&url)
        .json(&request)
        .send()
        .await
        .with_context(|| format!("Failed to start pulling model '{model_name}'"))?;

    if !response.status().is_success() {
        let status = response.status();
        let body = response.text().await.unwrap_or_default();
        anyhow::bail!("Failed to pull model '{model_name}' ({status}): {body}");
    }

    // Get response body as text for line-by-line parsing
    let body = response
        .text()
        .await
        .context("Failed to read pull response")?;

    // Create progress bar
    let pb = ProgressBar::new(100);
    pb.set_style(
        ProgressStyle::default_bar()
            .template("{spinner:.green} [{bar:40.cyan/blue}] {percent}% {msg}")
            .map_err(|e| anyhow::anyhow!("Failed to set progress style: {e}"))?
            .progress_chars("█▓░"),
    );

    let mut last_digest = String::new();
    let mut success = false;

    // Parse JSON lines
    for line in body.lines() {
        if line.trim().is_empty() {
            continue;
        }

        let progress: PullProgress = serde_json::from_str(line)
            .with_context(|| format!("Failed to parse progress: {line}"))?;

        match progress.status.as_str() {
            "success" => {
                success = true;
                pb.finish_with_message("Complete!");
            }
            "pulling manifest" => {
                pb.set_message("Pulling manifest...");
            }
            "verifying sha256 digest" => {
                pb.set_message("Verifying...");
            }
            "writing manifest" => {
                pb.set_message("Writing manifest...");
            }
            _ => {
                // Download progress
                if let (Some(total), Some(completed)) = (progress.total, progress.completed) {
                    if total > 0 {
                        let percent = completed * 100 / total;
                        pb.set_position(percent);

                        // Show new layer info
                        if let Some(ref digest) = progress.digest {
                            if digest != &last_digest {
                                last_digest.clone_from(digest);
                                let short_digest = &digest[..digest.len().min(12)];
                                pb.set_message(format!("Layer {short_digest}..."));
                            }
                        }
                    }
                }
            }
        }
    }

    if !success {
        anyhow::bail!("Pull did not complete successfully");
    }

    println!("✓ Model '{model_name}' ready");
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_recommended_model_serialization() {
        let model = RecommendedModel {
            name: "llama3.2:3b".to_string(),
            ram_gb: 4.0,
            speed_rating: 5,
            quality_rating: 3,
            installed: true,
        };

        let json = serde_json::to_string(&model).expect("Failed to serialize");
        assert!(json.contains("\"name\":\"llama3.2:3b\""));
        assert!(json.contains("\"ram_gb\":4.0"));
        assert!(json.contains("\"speed_rating\":5"));
        assert!(json.contains("\"quality_rating\":3"));
        assert!(json.contains("\"installed\":true"));
    }

    #[test]
    fn test_recommended_model_deserialization() {
        let json = r#"{
            "name": "mistral:7b",
            "ram_gb": 6.0,
            "speed_rating": 4,
            "quality_rating": 4,
            "installed": false
        }"#;

        let model: RecommendedModel = serde_json::from_str(json).expect("Failed to deserialize");
        assert_eq!(model.name, "mistral:7b");
        assert_eq!(model.ram_gb, 6.0);
        assert_eq!(model.speed_rating, 4);
        assert_eq!(model.quality_rating, 4);
        assert!(!model.installed);
    }

    #[test]
    fn test_default_models_content() {
        let models = get_default_models();

        assert_eq!(models.len(), 4);

        // Check first model
        assert_eq!(models[0].name, "llama3.2:3b");
        assert_eq!(models[0].ram_gb, 4.0);
        assert_eq!(models[0].speed_rating, 5);
        assert_eq!(models[0].quality_rating, 3);
        assert!(!models[0].installed);

        // Check all models have valid names
        for model in &models {
            assert!(!model.name.is_empty());
            assert!(model.speed_rating >= 1 && model.speed_rating <= 5);
            assert!(model.quality_rating >= 1 && model.quality_rating <= 5);
        }
    }

    #[test]
    fn test_model_equality() {
        let model1 = RecommendedModel {
            name: "test".to_string(),
            ram_gb: 4.0,
            speed_rating: 5,
            quality_rating: 3,
            installed: false,
        };

        let model2 = model1.clone();
        assert_eq!(model1, model2);
    }

    #[test]
    fn test_format_stars() {
        assert_eq!(format_stars(5), "★★★★★");
        assert_eq!(format_stars(3), "★★★☆☆");
        assert_eq!(format_stars(1), "★☆☆☆☆");
        assert_eq!(format_stars(0), "☆☆☆☆☆");
    }

    #[test]
    fn test_get_recommended_model_index() {
        // >= 8 GB -> llama3.1:8b (index 2)
        assert_eq!(get_recommended_model_index(16.0), 2);
        assert_eq!(get_recommended_model_index(8.0), 2);

        // >= 6 GB -> mistral:7b (index 1)
        assert_eq!(get_recommended_model_index(7.0), 1);
        assert_eq!(get_recommended_model_index(6.0), 1);

        // >= 4 GB -> llama3.2:3b (index 0)
        assert_eq!(get_recommended_model_index(5.0), 0);
        assert_eq!(get_recommended_model_index(4.0), 0);

        // < 4 GB -> phi3:mini (index 3)
        assert_eq!(get_recommended_model_index(3.0), 3);
        assert_eq!(get_recommended_model_index(2.0), 3);
    }

    #[test]
    fn test_get_system_ram_gb() {
        let ram = get_system_ram_gb();
        // RAM should be positive and reasonable (0.5 GB to 1 TB)
        assert!(ram > 0.5);
        assert!(ram < 1024.0);
    }

    #[test]
    fn test_pull_progress_deserialization() {
        let json = r#"{"status": "downloading", "digest": "sha256:abc123", "total": 2000000000, "completed": 500000000}"#;
        let progress: PullProgress = serde_json::from_str(json).expect("Failed to deserialize");

        assert_eq!(progress.status, "downloading");
        assert_eq!(progress.digest, Some("sha256:abc123".to_string()));
        assert_eq!(progress.total, Some(2_000_000_000));
        assert_eq!(progress.completed, Some(500_000_000));
    }

    #[test]
    fn test_pull_progress_minimal() {
        let json = r#"{"status": "success"}"#;
        let progress: PullProgress = serde_json::from_str(json).expect("Failed to deserialize");

        assert_eq!(progress.status, "success");
        assert_eq!(progress.digest, None);
        assert_eq!(progress.total, None);
        assert_eq!(progress.completed, None);
    }

    #[test]
    fn test_pull_progress_percentage_calculation() {
        let total: u64 = 2_000_000_000;
        let completed: u64 = 500_000_000;
        let percent = (completed * 100 / total) as u64;
        assert_eq!(percent, 25);
    }
}
