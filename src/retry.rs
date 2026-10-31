use std::time::Duration;

/// Capped exponential delay; never spins even after counter overflow.
pub fn delay(failures: u32) -> Duration {
    Duration::from_secs(1u64.checked_shl(failures).unwrap_or(30).min(30))
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn backoff_is_bounded() {
        let seconds: Vec<u64> = (0..8).map(|n| delay(n).as_secs()).collect();
        assert_eq!(seconds, [1, 2, 4, 8, 16, 30, 30, 30]);
        assert_eq!(delay(u32::MAX).as_secs(), 30);
    }
}
