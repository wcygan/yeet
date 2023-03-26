use std::time::Duration;
use tokio::time::Instant;

pub static EXPIRATION_TIME: Duration = Duration::from_secs(5);

pub fn next_instant(instant: Instant, duration: Duration) -> Instant {
    instant + duration
}
