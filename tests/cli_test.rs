//! Integration tests for CLI argument parsing.

use std::time::Duration;

use assert_cmd::Command;
use predicates::prelude::*;

/// Get a command for the ulm binary.
fn ulm() -> Command {
    Command::cargo_bin("ulm").expect("Failed to find ulm binary")
}

#[test]
fn test_help_flag() {
    ulm()
        .arg("--help")
        .assert()
        .success()
        .stdout(predicate::str::contains("transforms CLI interaction"))
        .stdout(predicate::str::contains("setup"))
        .stdout(predicate::str::contains("update"));
}

#[test]
fn test_version_flag() {
    ulm()
        .arg("--version")
        .assert()
        .success()
        .stdout(predicate::str::contains("ulm"))
        .stdout(predicate::str::contains("0.1.0"));
}

#[test]
fn test_setup_subcommand() {
    // Setup requires Ollama to be running, so we just check it starts correctly
    // Use timeout since it will fail waiting for Ollama
    ulm()
        .arg("setup")
        .timeout(Duration::from_secs(5))
        .assert()
        .stdout(predicate::str::contains(
            "ulm setup - Initializing manpage index",
        ));
}

#[test]
fn test_update_subcommand() {
    // Update processes all manpages which takes time, just check it starts
    // Use timeout since full processing takes too long
    ulm()
        .arg("update")
        .timeout(Duration::from_secs(5))
        .assert()
        .stdout(predicate::str::contains(
            "ulm update - Refreshing manpage index",
        ));
}

#[test]
fn test_query_string_capture() {
    ulm()
        .args(["find", "large", "files"])
        .assert()
        .success()
        .stdout(predicate::str::contains("Query: find large files"));
}

#[test]
fn test_query_with_quotes() {
    ulm()
        .arg("find large files in current directory")
        .assert()
        .success()
        .stdout(predicate::str::contains(
            "Query: find large files in current directory",
        ));
}

#[test]
fn test_no_args_shows_help_message() {
    ulm()
        .assert()
        .success()
        .stdout(predicate::str::contains(
            "ulm - AI-powered manpage assistant",
        ))
        .stdout(predicate::str::contains("--help"));
}

#[test]
fn test_invalid_subcommand() {
    ulm()
        .arg("invalid-command")
        .assert()
        .success() // It treats it as a query
        .stdout(predicate::str::contains("Query: invalid-command"));
}

#[test]
fn test_help_short_flag() {
    ulm()
        .arg("-h")
        .assert()
        .success()
        .stdout(predicate::str::contains("AI-powered manpage assistant"));
}

#[test]
fn test_version_short_flag() {
    ulm()
        .arg("-V")
        .assert()
        .success()
        .stdout(predicate::str::contains("0.1.0"));
}
