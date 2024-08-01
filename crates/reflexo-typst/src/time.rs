pub use instant::SystemTime as Time;

#[cfg(any(feature = "system", feature = "web"))]
pub fn now() -> Time {
    Time::now()
}

#[cfg(not(any(feature = "system", feature = "web")))]
pub fn now() -> Time {
    Time::UNIX_EPOCH
}
