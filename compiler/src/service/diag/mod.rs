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
