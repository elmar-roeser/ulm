//! Command execution and clipboard integration.
//!
//! This module handles executing commands and clipboard operations:
//! - Shell command execution
//! - Clipboard copy functionality

pub mod shell;

pub use shell::execute_command;
