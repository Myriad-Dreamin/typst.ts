// todo: remove cfg feature here
#[cfg(feature = "system-compile")]
mod console;
#[cfg(feature = "system-compile")]
pub use console::*;

/// Which format to use for diagnostics.
#[derive(Debug, Copy, Clone, Eq, PartialEq, Ord, PartialOrd)]
pub enum DiagnosticFormat {
    Human,
    Short,
}

impl Default for DiagnosticFormat {
    fn default() -> Self {
        Self::Human
    }
}

/// The status in which the watcher can be.
#[derive(Debug, Clone, Copy)]
pub enum DiagStatus {
    Stage(&'static str),
    Success(std::time::Duration),
    Error(std::time::Duration),
}
