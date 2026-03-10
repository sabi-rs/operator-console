# operator-console

Protocol-first Rust TUI for the `sabi` operator shell.

Current scope:
- Ratatui shell with `Dashboard` and `Exchanges` panels
- generic exchange-domain snapshot models
- typed provider contract
- transport-shaped stub exchange provider
- worker request/response scaffolding for the future Python exchange worker
- preserved `bet_recorder_provider` integration slice for the existing watcher subprocess

## Run

```bash
cd /home/thomas/projects/sabi/console/operator-console
cargo run
```

Core keys:
- `q` quit
- `tab` switch panel
- `j`/`k` move in the exchanges list
- `r` refresh

## Test

```bash
cd /home/thomas/projects/sabi/console/operator-console
cargo test
```

## Architecture

The shell does not read fixture files directly. The UI talks to a provider boundary, and the
default `StubExchangeProvider` simply loads a transport-shaped snapshot from
`fixtures/exchange_panel_snapshot.json`.

That keeps the Rust side aligned with the longer-term architecture:
- Rust owns terminal lifecycle, panels, routing, and operator state
- a worker-backed provider owns live data acquisition
- the Python exchange browser worker can replace the stub provider without a UI rewrite
