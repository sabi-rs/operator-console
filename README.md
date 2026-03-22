# operator-console

Rust Ratatui console for Sabi trading workflows.

It is the main operator interface for positions, calculators, matchers, recorder control, and observability over worker-backed betting data.

## Owns

- Trading-focused operator UX
- Panel routing, keyboard flow, and persisted local UI state
- Rendering recorder-backed positions, stats, matcher, calculator, and observability views

## Integrates With

- `../../bet-recorder` for normalized snapshots and recorder control
- `../../workers/exchange-browser-worker` indirectly through recorder-managed worker flows

If a change alters transport messages, recorder snapshot shape, or live recorder behavior, verify the console against the recorder change rather than treating this repo as isolated.

## Run

```bash
cargo run
```

## Test

```bash
cargo test
```

Target a narrower test when possible:

```bash
cargo test --test recorder_controls
cargo test recorder_start_and_stop_are_controllable_from_app
```
