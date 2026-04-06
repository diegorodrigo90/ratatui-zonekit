# CLAUDE.md

## Project

`ratatui-zonekit` — extensible zone and plugin rendering system for [ratatui](https://ratatui.rs).

## Commands

```bash
cargo build                 # Build
cargo test                  # Run all tests
cargo clippy --all-targets  # Lint (pedantic)
cargo fmt                   # Format
cargo doc --open            # Generate and view docs
```

## Architecture

```
src/
├── lib.rs          ← re-exports + doc example
├── zone.rs         ← ZoneId, ZoneHint, ZoneRequest, ZoneSpec
├── plugin.rs       ← ZonePlugin trait, RenderContext, ZoneEvent
├── registry.rs     ← ZoneRegistry (allocation, ownership, queries)
└── render.rs       ← SafeRenderer (catch_unwind isolation)
```

## Rules

- Theme-agnostic: works with any styling system (themekit, raw Style, custom)
- Model B: plugins declare, host renders. Plugins never touch Frame
- Safe delegation: catch_unwind around every plugin render
- `Send + Sync` required on ZonePlugin implementations
- Clippy pedantic — zero warnings
- RUSTFLAGS="-Dwarnings" on all CI and hooks
- Conventional commits
- `interactive: true` in lefthook for cargo commands
