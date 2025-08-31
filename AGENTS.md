# Repository Guidelines

## Project Structure & Module Organization
- Root: CI configs in `.github/`. App crate lives in `search_devices/`.
- Source: `search_devices/src/` with `main.rs`, `cidr_tab.rs`, `ip_list_tab.rs`, `tracert_tab.rs`, `utils.rs`.
- Build output: `search_devices/target/` (ignored). Docs artifacts are zipped in CI.

## Build, Test, and Development Commands
- Build (release): `cd search_devices && cargo build --release`
- Run (release): `cd search_devices && cargo run --release`
- Test: `cd search_devices && cargo test`
- Format check: `cd search_devices && cargo fmt -- --check`
- Lint: `cd search_devices && cargo clippy -- -D warnings`
Each command runs in the `search_devices` directory (the Cargo crate root).

## Coding Style & Naming Conventions
- Language: Rust 2021 edition.
- Formatting: `rustfmt` default settings; 4‑space indentation; line width default.
- Linting: Prefer `cargo clippy` clean (no warnings for changed code).
- Naming: `snake_case` for functions/vars, `CamelCase` for types, `SCREAMING_SNAKE_CASE` for consts. Modules map to files under `src/` (e.g., `tracert_tab.rs`).

## Testing Guidelines
- Framework: Rust built‑in test runner (`cargo test`).
- Scope: Keep logic testable in `utils.rs` and small helpers; avoid UI‑only tests.
- Conventions: Place module tests under `#[cfg(test)] mod tests { ... }` next to code. Name tests for the behavior (e.g., `test_traceroute_args_for_tracert_tab`).
- Run locally: `cargo test -q` from `search_devices/`.

## Commit & Pull Request Guidelines
- Commits: Short, imperative subject (English or Japanese OK). Group related changes; include scope when helpful (e.g., `tracert:`).
- Messages: Explain why when non‑obvious; reference issues (`Fixes #123`).
- PRs: Include description, rationale, screenshots/GIFs for UI, tested OS (Windows/Linux), and a brief test plan. Link issues and note any follow‑ups.

## Security & Configuration Tips
- External tools: Uses `ping`, `tracert` (Windows) or `traceroute` (Unix). Ensure they’re installed and runnable without elevated prompts.
- Timeouts: Unix `-W` is seconds; code converts ms→ceil(seconds). Validate with tests when changing.
- Windows GUI: Release builds hide the console via `windows_subsystem = "windows"`.

