/*!
agent-turn-limit: enforce a max-turn cap on LLM agent loops.

```rust
use agent_turn_limit::TurnLimit;

let mut limit = TurnLimit::new(3);
assert!(limit.check().is_ok());
limit.increment();
limit.increment();
limit.increment();
assert!(limit.check().is_err());
```
*/

use std::fmt;

/// Raised when the turn limit is exceeded.
#[derive(Debug, Clone, PartialEq)]
pub struct TurnLimitExceeded {
    pub current: usize,
    pub max: usize,
}

impl fmt::Display for TurnLimitExceeded {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "turn limit exceeded: {} / {}", self.current, self.max)
    }
}

impl std::error::Error for TurnLimitExceeded {}

/// A simple counter that raises when a max is hit.
#[derive(Debug, Clone)]
pub struct TurnLimit {
    max: usize,
    current: usize,
}

impl TurnLimit {
    /// Create a new limit with `max` allowed turns.
    pub fn new(max: usize) -> Self {
        Self { max, current: 0 }
    }

    /// Current turn count.
    pub fn current(&self) -> usize {
        self.current
    }

    /// Max turns allowed.
    pub fn max(&self) -> usize {
        self.max
    }

    /// Remaining turns before the limit is hit.
    pub fn remaining(&self) -> usize {
        self.max.saturating_sub(self.current)
    }

    /// True if the limit has been reached or exceeded.
    pub fn is_exceeded(&self) -> bool {
        self.current >= self.max
    }

    /// Increment turn count by one.
    ///
    /// The count saturates at [`usize::MAX`], so this never panics even in an
    /// unbounded loop running for an extremely long time.
    pub fn increment(&mut self) {
        self.current = self.current.saturating_add(1);
    }

    /// Increment and immediately check — returns Err if limit now exceeded.
    pub fn tick(&mut self) -> Result<usize, TurnLimitExceeded> {
        self.increment();
        self.check()?;
        Ok(self.current)
    }

    /// Check whether the limit is exceeded without incrementing.
    pub fn check(&self) -> Result<(), TurnLimitExceeded> {
        if self.is_exceeded() {
            Err(TurnLimitExceeded {
                current: self.current,
                max: self.max,
            })
        } else {
            Ok(())
        }
    }

    /// Reset to zero.
    pub fn reset(&mut self) {
        self.current = 0;
    }

    /// Fraction of budget consumed [0.0, 1.0].
    pub fn fraction_used(&self) -> f64 {
        if self.max == 0 {
            1.0
        } else {
            self.current as f64 / self.max as f64
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn starts_at_zero() {
        let l = TurnLimit::new(5);
        assert_eq!(l.current(), 0);
        assert_eq!(l.max(), 5);
    }

    #[test]
    fn remaining_full() {
        let l = TurnLimit::new(5);
        assert_eq!(l.remaining(), 5);
    }

    #[test]
    fn increment_advances() {
        let mut l = TurnLimit::new(5);
        l.increment();
        l.increment();
        assert_eq!(l.current(), 2);
        assert_eq!(l.remaining(), 3);
    }

    #[test]
    fn not_exceeded_under_limit() {
        let l = TurnLimit::new(5);
        assert!(!l.is_exceeded());
        assert!(l.check().is_ok());
    }

    #[test]
    fn exceeded_at_limit() {
        let mut l = TurnLimit::new(2);
        l.increment();
        l.increment();
        assert!(l.is_exceeded());
        assert!(l.check().is_err());
    }

    #[test]
    fn tick_ok_under_limit() {
        let mut l = TurnLimit::new(3);
        let n = l.tick().unwrap();
        assert_eq!(n, 1);
    }

    #[test]
    fn tick_err_at_limit() {
        let mut l = TurnLimit::new(2);
        l.tick().unwrap();
        assert!(l.tick().is_err());
    }

    #[test]
    fn error_display() {
        let e = TurnLimitExceeded { current: 5, max: 3 };
        assert!(e.to_string().contains("5"));
        assert!(e.to_string().contains("3"));
    }

    #[test]
    fn reset_clears_count() {
        let mut l = TurnLimit::new(3);
        l.increment();
        l.increment();
        l.reset();
        assert_eq!(l.current(), 0);
        assert!(!l.is_exceeded());
    }

    #[test]
    fn fraction_used_zero() {
        let l = TurnLimit::new(10);
        assert_eq!(l.fraction_used(), 0.0);
    }

    #[test]
    fn fraction_used_half() {
        let mut l = TurnLimit::new(10);
        for _ in 0..5 {
            l.increment();
        }
        assert!((l.fraction_used() - 0.5).abs() < 1e-9);
    }

    #[test]
    fn fraction_used_full() {
        let mut l = TurnLimit::new(4);
        for _ in 0..4 {
            l.increment();
        }
        assert!((l.fraction_used() - 1.0).abs() < 1e-9);
    }

    #[test]
    fn zero_max_is_immediately_exceeded() {
        let l = TurnLimit::new(0);
        assert!(l.is_exceeded());
    }

    #[test]
    fn remaining_saturates_at_zero() {
        let mut l = TurnLimit::new(2);
        for _ in 0..5 {
            l.increment();
        }
        assert_eq!(l.remaining(), 0);
    }

    #[test]
    fn clone_is_independent() {
        let mut l = TurnLimit::new(5);
        let l2 = l.clone();
        l.increment();
        assert_eq!(l.current(), 1);
        assert_eq!(l2.current(), 0);
    }

    #[test]
    fn increment_saturates_at_usize_max() {
        let mut l = TurnLimit::new(usize::MAX);
        l.current = usize::MAX;
        // Must not panic on overflow; the count stays pinned at the ceiling.
        l.increment();
        assert_eq!(l.current(), usize::MAX);
    }

    #[test]
    fn fraction_used_zero_max_is_one() {
        // A zero budget is always fully consumed and avoids division by zero.
        let l = TurnLimit::new(0);
        assert_eq!(l.fraction_used(), 1.0);
    }

    #[test]
    fn tick_returns_running_count() {
        let mut l = TurnLimit::new(3);
        assert_eq!(l.tick().unwrap(), 1);
        assert_eq!(l.tick().unwrap(), 2);
    }

    #[test]
    fn check_does_not_increment() {
        let l = TurnLimit::new(3);
        assert!(l.check().is_ok());
        assert!(l.check().is_ok());
        assert_eq!(l.current(), 0);
    }

    #[test]
    fn error_carries_counts() {
        let mut l = TurnLimit::new(1);
        l.increment();
        let err = l.check().unwrap_err();
        assert_eq!(err.current, 1);
        assert_eq!(err.max, 1);
    }

    #[test]
    fn reset_allows_reuse_after_exceeded() {
        let mut l = TurnLimit::new(2);
        l.increment();
        l.increment();
        assert!(l.is_exceeded());
        l.reset();
        assert!(!l.is_exceeded());
        assert!(l.tick().is_ok());
    }
}
