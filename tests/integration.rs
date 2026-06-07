//! Integration tests that drive `TurnLimit` through the public API only,
//! mirroring how a caller would use it inside an agent loop.

use agent_turn_limit::{TurnLimit, TurnLimitExceeded};

/// Simulate an agent loop that stops once the turn budget is exhausted.
///
/// `tick` increments first, then checks, so the tick that pushes the counter up
/// to `max` is the one that errors. A `tick`-driven loop therefore runs
/// `max - 1` successful turns before stopping.
#[test]
fn agent_loop_stops_at_budget() {
    let mut limit = TurnLimit::new(4);
    let mut turns_taken = 0;

    loop {
        if limit.tick().is_err() {
            break;
        }
        turns_taken += 1;
    }

    assert_eq!(turns_taken, 3);
    assert!(limit.is_exceeded());
    assert_eq!(limit.remaining(), 0);
    assert_eq!(limit.current(), 4);
}

/// `check` should gate work *before* a turn is spent, leaving the count alone.
#[test]
fn check_before_work_does_not_consume_budget() {
    let limit = TurnLimit::new(2);
    for _ in 0..10 {
        assert!(limit.check().is_ok());
    }
    assert_eq!(limit.current(), 0);
    assert_eq!(limit.remaining(), 2);
}

#[test]
fn progress_reporting_tracks_fraction() {
    let mut limit = TurnLimit::new(5);
    assert_eq!(limit.fraction_used(), 0.0);
    limit.increment();
    limit.increment();
    assert!((limit.fraction_used() - 0.4).abs() < 1e-9);
}

#[test]
fn reset_reuses_the_same_limiter() {
    let mut limit = TurnLimit::new(3);
    assert!(limit.tick().is_ok()); // current = 1
    assert!(limit.tick().is_ok()); // current = 2
    assert!(limit.tick().is_err()); // current = 3, hits the cap

    limit.reset();
    assert_eq!(limit.current(), 0);
    assert!(limit.tick().is_ok());
}

#[test]
fn exceeded_error_is_a_std_error() {
    let mut limit = TurnLimit::new(1);
    limit.increment();
    let boxed: Box<dyn std::error::Error> = Box::new(limit.check().unwrap_err());
    assert!(boxed.to_string().contains("turn limit exceeded"));
}

#[test]
fn error_equality_and_fields() {
    let a = TurnLimitExceeded { current: 3, max: 2 };
    let b = TurnLimitExceeded { current: 3, max: 2 };
    assert_eq!(a, b);
    assert_eq!(a.current, 3);
    assert_eq!(a.max, 2);
}
