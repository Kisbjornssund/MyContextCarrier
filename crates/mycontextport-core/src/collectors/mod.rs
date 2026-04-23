//! Built-in context collectors.

pub mod clipboard;
pub mod python_bridge;
pub mod shell_history;

pub use clipboard::ClipboardCollector;
pub use python_bridge::PythonCollector;
pub use shell_history::ShellHistoryCollector;
