pub use web_time::Duration;
pub use web_time::Instant;
pub use web_time::SystemTime as Time;

/// Returns the current time (UTC+0).
#[cfg(any(feature = "system", feature = "web"))]
pub fn now() -> Time {
    Time::now()
}

/// Returns a dummy time on environments that do not support time.
#[cfg(not(any(feature = "system", feature = "web")))]
pub fn now() -> Time {
    Time::UNIX_EPOCH
}
