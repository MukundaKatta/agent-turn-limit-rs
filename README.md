# agent-turn-limit

A tiny, dependency-free Rust crate that enforces a maximum-turn cap on LLM
agent loops. Use it as a guardrail to stop runaway agents that would otherwise
loop indefinitely, burning tokens and money.

The core type, `TurnLimit`, is a small counter that tracks how many turns an
agent has taken against a configured maximum and signals (via `Result` or
boolean checks) when the budget is exhausted.

## Features

- Zero dependencies — pure standard library.
- Simple counter API: `increment`, `tick`, `check`, `reset`.
- Inspect state at any time: `current`, `max`, `remaining`, `is_exceeded`,
  `fraction_used`.
- A typed error, `TurnLimitExceeded`, that implements `std::error::Error` and
  `Display`.

## Installation

Add the crate to your `Cargo.toml`:

```toml
[dependencies]
agent-turn-limit = "0.1"
```

## Usage

Create a limit with the maximum number of allowed turns, then call `tick()` at
the top of each agent iteration. `tick` increments the counter and returns an
error once the cap is reached:

```rust
use agent_turn_limit::TurnLimit;

let mut limit = TurnLimit::new(3);

loop {
    match limit.tick() {
        Ok(turn) => {
            println!("running turn {turn}");
            // ... run one agent step ...
        }
        Err(e) => {
            eprintln!("stopping: {e}");
            break;
        }
    }
}
```

You can also check the budget without incrementing, and inspect remaining
capacity:

```rust
use agent_turn_limit::TurnLimit;

let mut limit = TurnLimit::new(5);
limit.increment();

assert_eq!(limit.current(), 1);
assert_eq!(limit.remaining(), 4);
assert!(!limit.is_exceeded());
assert!(limit.check().is_ok());
assert!((limit.fraction_used() - 0.2).abs() < 1e-9);

limit.reset();
assert_eq!(limit.current(), 0);
```

## API overview

| Method | Description |
| --- | --- |
| `TurnLimit::new(max)` | Create a limit allowing `max` turns. |
| `current()` | Current turn count. |
| `max()` | Maximum turns allowed. |
| `remaining()` | Turns left before the limit (saturates at 0). |
| `is_exceeded()` | `true` once `current >= max`. |
| `increment()` | Increment the turn count by one. |
| `tick()` | Increment, then return `Err` if the limit is now exceeded. |
| `check()` | Return `Err` if exceeded, without incrementing. |
| `reset()` | Reset the count back to zero. |
| `fraction_used()` | Fraction of the budget consumed, in `[0.0, 1.0]`. |

A limit created with `max = 0` is considered exceeded immediately.

## Building and testing

```bash
cargo build
cargo test
```

## License

Licensed under the MIT License. See the `license` field in `Cargo.toml`.
