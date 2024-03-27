use std::str::FromStr;

use ::tracing::metadata::LevelFilter;
use tracing::dispatcher;
use tracing_error::ErrorLayer;
use tracing_subscriber::{fmt, prelude::*, Layer};

#[allow(unused)]
pub struct TraceGuard(dispatcher::DefaultGuard);

/// A guard for the tracing subscriber.
/// When dropped, dumps the trace to somewhere.
impl TraceGuard {
    /// New with option format: verbosity={0..3}
    pub fn new(options: String) -> Result<Self, String> {
        let level = level_filter(options)?;

        // Build the FMT layer printing to the console.
        let fmt_layer = fmt::Layer::default().with_filter(level);

        // Error layer for building backtraces
        let error_layer = ErrorLayer::default();

        // Build the registry.
        let registry = tracing_subscriber::registry()
            .with(fmt_layer)
            .with(error_layer);

        Ok(Self(registry.set_default()))
    }
}

/// Returns the log level filter for the given verbosity level.
fn level_filter(args: String) -> Result<LevelFilter, String> {
    if let Some(res) = args.strip_prefix("verbosity=") {
        return Ok(
            match res
                .parse()
                .map_err(|e: <i64 as FromStr>::Err| e.to_string())?
            {
                0 => LevelFilter::WARN,
                1 => LevelFilter::INFO,
                2 => LevelFilter::DEBUG,
                _ => LevelFilter::TRACE,
            },
        );
    }

    Ok(LevelFilter::TRACE)
}
