// todo: remove cfg feature here
#[cfg(feature = "system-compile")]
mod console;
#[cfg(feature = "system-compile")]
pub use console::*;

/// Which format to use for diagnostics.
#[derive(Debug, Copy, Clone, Default, Eq, PartialEq, Ord, PartialOrd)]
pub enum DiagnosticFormat {
    #[default]
    Human,
    Short,
}
