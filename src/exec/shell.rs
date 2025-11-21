//! Shell command execution.
//!
//! This module handles executing commands in the user's shell.

use std::process::Command;

use anyhow::{Context, Result};
use tracing::{debug, info};

/// Executes a command in the user's shell.
///
/// Spawns the command using `sh -c` and inherits stdin/stdout/stderr.
/// Returns the exit code of the command.
///
/// # Errors
///
/// Returns an error if:
/// - The shell cannot be spawned
/// - The command cannot be executed
pub fn execute_command(command: &str) -> Result<i32> {
    info!(command = %command, "Executing command");

    let mut child = Command::new("sh")
        .arg("-c")
        .arg(command)
        .stdin(std::process::Stdio::inherit())
        .stdout(std::process::Stdio::inherit())
        .stderr(std::process::Stdio::inherit())
        .spawn()
        .with_context(|| format!("Failed to spawn command: {command}"))?;

    let status = child
        .wait()
        .with_context(|| format!("Failed to wait for command: {command}"))?;

    let exit_code = status.code().unwrap_or(1);

    debug!(exit_code = exit_code, "Command completed");

    Ok(exit_code)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_execute_true() {
        let result = execute_command("true");
        assert!(result.is_ok());
        assert_eq!(result.unwrap_or(1), 0);
    }

    #[test]
    fn test_execute_false() {
        let result = execute_command("false");
        assert!(result.is_ok());
        assert_ne!(result.unwrap_or(0), 0);
    }

    #[test]
    fn test_execute_echo() {
        let result = execute_command("echo test > /dev/null");
        assert!(result.is_ok());
        assert_eq!(result.unwrap_or(1), 0);
    }

    #[test]
    fn test_execute_exit_code() {
        let result = execute_command("exit 42");
        assert!(result.is_ok());
        assert_eq!(result.unwrap_or(0), 42);
    }
}
