# AGENTS.md

Operational contract for AI coding assistants working on this repository.

## Golden Rules

1. Theme-agnostic — ZERO dependency on any specific theme crate
2. Model B — plugins declare intent, host renders. Plugins never access Frame
3. Safe delegation — every plugin render wrapped in catch_unwind
4. `Send + Sync` required on all ZonePlugin implementations
5. Every public item has doc comments
6. Clippy pedantic — zero warnings with -D warnings
7. RUSTFLAGS="-Dwarnings" on cargo test (same as CI)
8. Conventional commits: `type(scope): description`
9. `interactive: true` in lefthook for all cargo commands

## Quality Gates (before commit)

```bash
cargo +nightly fmt --check
RUSTFLAGS="-Dwarnings" cargo clippy --all-targets -- -D warnings
RUSTFLAGS="-Dwarnings" cargo test
cargo doc --no-deps
```
