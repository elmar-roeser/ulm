//! Command execution and clipboard integration.
//!
//! This module handles executing commands and clipboard operations:
//! - Shell command execution
//! - Clipboard copy functionality
//! - Command editing

pub mod clipboard;
pub mod edit;
pub mod shell;

pub use clipboard::copy_to_clipboard;
pub use edit::edit_command;
pub use shell::execute_command;
