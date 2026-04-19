# pm-asana

Asana adapter for [pm-sdk-rs](../../README.md), implementing the `PmAdapter` trait.

## Authentication

Set the `ASANA_PAT` environment variable to your [Personal Access Token](https://app.asana.com/0/my-apps).

```bash
export ASANA_PAT=your_personal_access_token
export ASANA_WORKSPACE_GID=your_workspace_gid  # required for create/search
```

## Usage

```rust
use pm_asana::AsanaAdapter;
use pm_core::*;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let adapter = AsanaAdapter::from_env();

    // List tasks
    let tasks = adapter.list_issues(PageRequest {
        limit: Some(20),
        ..Default::default()
    }).await?;

    for task in &tasks {
        println!("[{}] {} — {}", task.id, task.title,
            task.status.as_deref().unwrap_or("unknown"));
    }

    // Create a task
    let task = adapter.create_issue(CreateIssueRequest {
        title: "New task".into(),
        description: Some("Task notes".into()),
        project_id: Some("project_gid".into()),
        ..Default::default()
    }).await?;

    // Complete a task
    adapter.update_issue(&task.id, UpdateIssueRequest {
        status: Some("completed".into()),
        ..Default::default()
    }).await?;

    // Delete a task
    adapter.delete_issue(&task.id).await?;

    // Tag CRUD (Asana-specific)
    let tags = adapter.list_tags(PageRequest::default()).await?;
    let tag = adapter.create_tag("backend").await?;
    let tag = adapter.get_tag(&tag.id).await?;
    adapter.delete_tag(&tag.id).await?;

    // Project CRUD
    let project = adapter.create_project("My Project", Some("notes")).await?;
    let project = adapter.update_project(&project.id, Some("Renamed"), None).await?;
    adapter.delete_project(&project.id).await?;

    // Search
    let results = adapter.search(SearchRequest {
        query: "auth".into(),
        limit: Some(10),
    }).await?;

    Ok(())
}
```

## Mapping

| Asana | pm-core |
|-------|---------|
| Task | `Issue` |
| Project | `Project` |
| Workspace | `Team` (via `list_teams`) |
| Story (comment) | `Comment` |
| Tag | `Team` (via `list_tags` / `create_tag`) |

## Asana-specific methods

Beyond the `PmAdapter` trait, `AsanaAdapter` exposes:

- `list_tags(page)` / `get_tag(gid)` / `create_tag(name)` / `delete_tag(gid)` — Tag CRUD
- `create_project(name, notes)` / `update_project(gid, name, notes)` / `delete_project(gid)` — Project write operations
- `with_workspace(gid)` — set workspace GID for task/project creation and search
- `from_env()` — read `ASANA_PAT` + `ASANA_WORKSPACE_GID` from environment

## Running the example

```bash
export ASANA_PAT=...
export ASANA_WORKSPACE_GID=...
cargo run --example basic_usage -p pm-asana
```
