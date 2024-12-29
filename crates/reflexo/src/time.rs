pub use instant::Duration;
pub use instant::Instant;
pub use instant::SystemTime as Time;

/// Returns the UTC+0 time.
#[cfg(any(feature = "system", feature = "web"))]
pub fn now() -> Time {
    Time::now()
}

/// Returns a dummy time on environments that do not support time.
#[cfg(not(any(feature = "system", feature = "web")))]
pub fn now() -> Time {
    Time::UNIX_EPOCH
}
