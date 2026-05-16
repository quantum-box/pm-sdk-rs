# pm-sdk-rs

[![CI](https://github.com/quantum-box/pm-sdk-rs/actions/workflows/ci.yml/badge.svg)](https://github.com/quantum-box/pm-sdk-rs/actions/workflows/ci.yml)
[![License: MIT](https://img.shields.io/badge/License-MIT-yellow.svg)](LICENSE)

Rust SDK for project management platforms (Linear, Asana).

## Crates

| Crate | Description |
|-------|-------------|
| `pm-core` | Platform-agnostic traits and domain models |
| `pm-linear` | Linear GraphQL adapter |
| `pm-asana` | Asana REST adapter |
| `pm-cli` | CLI tool (`pm-sdk`) |

## Installation

Use this repository as a Git dependency. Add the adapter you need to your
`Cargo.toml`. Pin `branch`, `tag`, or `rev` in production applications when you
need reproducible builds.

```toml
[dependencies]
pm-core = { git = "https://github.com/quantum-box/pm-sdk-rs", branch = "main" }
pm-linear = { git = "https://github.com/quantum-box/pm-sdk-rs", branch = "main" } # for Linear
pm-asana = { git = "https://github.com/quantum-box/pm-sdk-rs", branch = "main" }  # for Asana
```

Install the CLI directly from GitHub:

```bash
cargo install --git https://github.com/quantum-box/pm-sdk-rs --locked pm-cli
```

Or build from source:

```bash
git clone https://github.com/quantum-box/pm-sdk-rs.git
cd pm-sdk-rs
cargo build --workspace
cargo run -p pm-cli -- --help
```

## Quick Start

### Linear

```rust
use pm_linear::LinearAdapter;
use pm_core::{PmAdapter, PageRequest};

#[tokio::main]
async fn main() {
    let adapter = LinearAdapter::new(&std::env::var("LINEAR_API_KEY").unwrap());
    let issues = adapter.list_issues(PageRequest::default()).await.unwrap();
    for issue in issues {
        println!("{}: {}", issue.id, issue.title);
    }
}
```

### Asana

```rust
use pm_asana::AsanaAdapter;
use pm_core::{PmAdapter, PageRequest};

#[tokio::main]
async fn main() {
    // reads ASANA_PAT and ASANA_WORKSPACE_GID from environment
    let adapter = AsanaAdapter::from_env();
    let tasks = adapter.list_issues(PageRequest::default()).await.unwrap();
    for task in tasks {
        println!("{}: {}", task.id, task.title);
    }
}
```

### CLI

```bash
export LINEAR_API_KEY=lin_api_xxx
pm-sdk list-issues --limit 10
pm-sdk create-issue --title "New feature"
pm-sdk search --query "auth bug"
```

## Environment Variables

| Variable | Required | Description |
|----------|----------|-------------|
| `LINEAR_API_KEY` | Linear only | Linear personal API key |
| `ASANA_PAT` | Asana only | Asana Personal Access Token |
| `ASANA_WORKSPACE_GID` | Asana (search) | Asana workspace GID |

## Examples

Runnable examples are available in [`examples/examples`](examples/examples).

```bash
cargo run -p examples --example linear_list_issues
cargo run -p examples --example asana_list_tasks
```

See [CONTRIBUTING.md](CONTRIBUTING.md) for local development commands.

## License

MIT — see [LICENSE](LICENSE).
