# pm-sdk-rs

[![Crates.io](https://img.shields.io/crates/v/pm-core.svg)](https://crates.io/crates/pm-core)
[![License: MIT](https://img.shields.io/badge/License-MIT-yellow.svg)](LICENSE)

Rust SDK for project management platforms (Linear, Asana).

## Crates

| Crate | Description | crates.io |
|-------|-------------|-----------|
| `pm-core` | Platform-agnostic traits and domain models | [![](https://img.shields.io/crates/v/pm-core.svg)](https://crates.io/crates/pm-core) |
| `pm-linear` | Linear GraphQL adapter | [![](https://img.shields.io/crates/v/pm-linear.svg)](https://crates.io/crates/pm-linear) |
| `pm-asana` | Asana REST adapter | [![](https://img.shields.io/crates/v/pm-asana.svg)](https://crates.io/crates/pm-asana) |
| `pm-cli` | CLI tool (`pm-sdk`) | [![](https://img.shields.io/crates/v/pm-cli.svg)](https://crates.io/crates/pm-cli) |

## Installation

Add the adapter you need to your `Cargo.toml`:

```toml
[dependencies]
pm-core = "0.1"
pm-linear = "0.1"   # for Linear
pm-asana  = "0.1"   # for Asana
```

Or install the CLI:

```bash
cargo install pm-cli
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

## License

MIT — see [LICENSE](LICENSE).
