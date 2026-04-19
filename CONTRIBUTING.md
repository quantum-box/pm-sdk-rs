# Contributing

## Prerequisites

- Rust stable (latest)
- `cargo clippy` and `cargo fmt` must pass

## Development

```bash
git clone https://github.com/quantum-box/pm-sdk-rs.git
cd pm-sdk-rs
cargo build --all
cargo test --all
```

## Adding a new adapter

1. Create a new crate under `crates/` following the pattern in `pm-linear` or `pm-asana`.
2. Implement `PmAdapter` from `pm-core`.
3. Add the crate to the workspace `Cargo.toml`.
4. Add a usage example to `examples/`.

## Pull Requests

- One logical change per PR.
- `cargo clippy --all -- -D warnings` must pass.
- `cargo fmt --all -- --check` must pass.
- Tests must pass: `cargo test --all`.

## License

By contributing, you agree that your contributions will be licensed under the MIT License.
