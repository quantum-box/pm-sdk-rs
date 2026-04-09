# pm-sdk-rs

Rust SDK for project management platforms (Linear, Jira).

## Crates

- **pm-core**: Platform-agnostic trait + domain models
- **pm-linear**: Linear GraphQL adapter
- **pm-cli**: CLI tool (`pm-sdk`)

## Usage

```bash
export LINEAR_API_KEY=lin_api_xxx
pm-sdk list-issues --limit 10
pm-sdk create-issue --title "New feature"
pm-sdk search --query "auth bug"
```
