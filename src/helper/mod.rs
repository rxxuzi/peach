// Helper modules for error handling and utilities
// Provides clean, developer-friendly error messages

pub mod types;
pub mod msg;
pub mod utils;

// Re-export commonly used items
pub use types::{CompileError, ErrorType, Span};
pub use msg::MessageFormatter;
pub use utils::{levenshtein_distance, find_similar};