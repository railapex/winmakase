//! Exponential restart backoff, per component.
//!
//! 1s, 2s, 4s, 8s, 16s, then capped at 30s. A component that stays up for
//! `healthy_reset` is treated as recovered and starts again from 1s — otherwise
//! a machine up for a week would take half a minute to recover from its first
//! crash.

use std::time::Duration;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Backoff {
    initial: Duration,
    max: Duration,
    failures: u32,
}

impl Backoff {
    pub fn new(initial: Duration, max: Duration) -> Self {
        Self {
            initial: initial.max(Duration::from_millis(1)),
            max: max.max(initial),
            failures: 0,
        }
    }

    /// The delay for the next restart, and count this failure.
    pub fn next_delay(&mut self) -> Duration {
        let delay = self
            .initial
            .checked_mul(1u32.checked_shl(self.failures).unwrap_or(u32::MAX))
            .unwrap_or(self.max)
            .min(self.max);
        self.failures = self.failures.saturating_add(1);
        delay
    }

    pub fn reset(&mut self) {
        self.failures = 0;
    }

    pub fn is_reset(&self) -> bool {
        self.failures == 0
    }

    pub fn failures(&self) -> u32 {
        self.failures
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn default_backoff() -> Backoff {
        Backoff::new(Duration::from_secs(1), Duration::from_secs(30))
    }

    fn secs(b: &mut Backoff, n: usize) -> Vec<u64> {
        (0..n).map(|_| b.next_delay().as_secs()).collect()
    }

    #[test]
    fn doubles_then_caps_at_thirty() {
        let mut b = default_backoff();
        assert_eq!(secs(&mut b, 8), vec![1, 2, 4, 8, 16, 30, 30, 30]);
    }

    #[test]
    fn recovery_starts_over_at_one_second() {
        let mut b = default_backoff();
        secs(&mut b, 4);
        assert!(!b.is_reset());
        b.reset();
        assert!(b.is_reset());
        assert_eq!(secs(&mut b, 3), vec![1, 2, 4]);
    }

    #[test]
    fn a_long_crash_loop_does_not_overflow() {
        let mut b = default_backoff();
        for _ in 0..10_000 {
            assert!(b.next_delay() <= Duration::from_secs(30));
        }
        assert_eq!(b.next_delay(), Duration::from_secs(30));
    }

    #[test]
    fn honours_configured_bounds() {
        let mut b = Backoff::new(Duration::from_millis(50), Duration::from_millis(200));
        let ms: Vec<u128> = (0..5).map(|_| b.next_delay().as_millis()).collect();
        assert_eq!(ms, vec![50, 100, 200, 200, 200]);
    }
}
