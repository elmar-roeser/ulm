//! Command execution and clipboard integration.
//!
//! This module handles executing commands and clipboard operations:
//! - Shell command execution
//! - Clipboard copy functionality

pub mod clipboard;
pub mod shell;

pub use clipboard::copy_to_clipboard;
pub use shell::execute_command;
