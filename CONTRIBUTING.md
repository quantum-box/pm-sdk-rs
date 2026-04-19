# Contributing

## Prerequisites

- Rust stable (latest) — install via [rustup](https://rustup.rs/)
- `cargo-nextest` for running tests (`cargo install cargo-nextest`)

## Setup

```bash
git clone https://github.com/quantum-box/pm-sdk-rs.git
cd pm-sdk-rs
rustup show   # installs toolchain from rust-toolchain.toml if present
```

## Build

```bash
cargo build --workspace
```

## Test

```bash
# preferred
cargo nextest run --workspace

# fallback if nextest is not installed
cargo test --workspace
```

## Lint & Format

```bash
cargo clippy --workspace -- -D warnings
cargo fmt --all
```

Both must pass before opening a PR.

## Adding a new adapter

1. Create a new crate under `crates/` following the pattern in `pm-linear` or `pm-asana`.
2. Implement `PmAdapter` from `pm-core`.
3. Add the crate to the workspace `Cargo.toml`.
4. Add a usage example under `examples/examples/`.

## Pull Requests

- Branch name: `feature/<short-description>` (e.g. `feature/plt-123-add-notion-adapter`)
- PR title must follow [Conventional Commits](https://www.conventionalcommits.org/):
  `<type>(<scope>): <description>` — e.g. `feat(pm-linear): add team filter support`
- One logical change per PR.
- `cargo clippy --workspace -- -D warnings` must pass.
- `cargo fmt --all -- --check` must pass.
- Tests must pass: `cargo nextest run --workspace`.

## License

By contributing, you agree that your contributions will be licensed under the MIT License.
