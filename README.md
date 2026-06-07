# agent-turn-limit

[![CI](https://github.com/MukundaKatta/agent-turn-limit-rs/actions/workflows/ci.yml/badge.svg)](https://github.com/MukundaKatta/agent-turn-limit-rs/actions/workflows/ci.yml)
[![License: MIT](https://img.shields.io/badge/license-MIT-blue.svg)](LICENSE)

A tiny, dependency-free guardrail that enforces a maximum number of turns in an
LLM agent loop.

Autonomous agents run in a loop: think, call a tool, observe, repeat. Without a
hard cap, a buggy prompt or a model that keeps "trying one more thing" can spin
forever, burning tokens and money. `TurnLimit` is a small counter with an
ergonomic API to stop that loop deterministically.

- Zero dependencies, `no_std`-friendly logic (uses only `std::fmt`/`std::error`).
- Saturating counter that never panics, even in an unbounded loop.
- Clear, typed error (`TurnLimitExceeded`) that implements `std::error::Error`.
- Helpers for budget reporting: `remaining()`, `fraction_used()`.

## Install

Add it to your `Cargo.toml`:

```toml
[dependencies]
agent-turn-limit = "0.1"
```

Or with cargo:

```sh
cargo add agent-turn-limit
```

## Usage

The most common pattern is to `tick()` once per turn and break out of the loop
when it returns an error:

```rust
use agent_turn_limit::TurnLimit;

fn run_agent() {
    let mut limit = TurnLimit::new(10);

    loop {
        // Spend one turn. `tick` increments first, then checks the cap.
        match limit.tick() {
            Ok(turn) => {
                println!("turn {turn} of {}", limit.max());
                // ... call the model, run a tool, observe the result ...
            }
            Err(e) => {
                eprintln!("stopping: {e}"); // "turn limit exceeded: 10 / 10"
                break;
            }
        }
    }
}
```

`tick()` increments first and then checks, so the call that pushes the counter
up to `max` is the one that returns `Err`. A `tick`-driven loop like the one
above therefore runs `max - 1` successful turns before stopping.

If you would rather gate work *before* spending a turn (and run the full `max`
turns), use `check()` and `increment()` separately:

```rust
use agent_turn_limit::TurnLimit;

let mut limit = TurnLimit::new(3);
while limit.check().is_ok() {
    // ... do one unit of work ...
    limit.increment();
}
assert!(limit.is_exceeded());
```

You can also report progress while the loop runs:

```rust
use agent_turn_limit::TurnLimit;

let mut limit = TurnLimit::new(20);
limit.increment();
limit.increment();
println!("{} turns left ({:.0}% used)",
    limit.remaining(),            // 18
    limit.fraction_used() * 100.0 // 10%
);
```

## API

`TurnLimit`

| Method | Returns | Description |
| --- | --- | --- |
| `new(max)` | `TurnLimit` | Create a limit allowing `max` turns. |
| `current()` | `usize` | Turns taken so far. |
| `max()` | `usize` | Configured maximum. |
| `remaining()` | `usize` | Turns left before the cap (saturates at 0). |
| `is_exceeded()` | `bool` | `true` once `current >= max`. |
| `increment()` | `()` | Add one turn. Saturates at `usize::MAX`; never panics. |
| `tick()` | `Result<usize, TurnLimitExceeded>` | Increment, then error if the cap is now exceeded. |
| `check()` | `Result<(), TurnLimitExceeded>` | Error if the cap is exceeded, without incrementing. |
| `reset()` | `()` | Reset the count to zero so the limiter can be reused. |
| `fraction_used()` | `f64` | Budget consumed in `[0.0, 1.0]`; a `max` of `0` returns `1.0`. |

`TurnLimitExceeded` is the error type. It exposes `current` and `max` fields,
implements `Display` (`"turn limit exceeded: {current} / {max}"`) and
`std::error::Error`, and derives `Debug`, `Clone`, and `PartialEq`.

### Edge cases

- `TurnLimit::new(0)` is exceeded immediately and `fraction_used()` is `1.0`.
- `remaining()` saturates at `0` rather than underflowing.
- `increment()` saturates at `usize::MAX` rather than overflowing/panicking.

## Development

```sh
cargo build
cargo test          # unit + integration + doc tests
cargo fmt --check
cargo clippy --all-targets -- -D warnings
```

## License

Licensed under the [MIT License](LICENSE).
