/// Basic usage example for pm-asana.
///
/// Set environment variables before running:
///   ASANA_PAT=your_personal_access_token
///   ASANA_WORKSPACE_GID=your_workspace_gid
///
///   cargo run --example basic_usage -p pm-asana
use pm_asana::AsanaAdapter;
use pm_core::*;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let adapter = AsanaAdapter::from_env();

    println!("Platform: {}", adapter.platform());

    // List projects
    let projects = adapter.list_projects(PageRequest::default()).await?;
    println!("\n=== Projects ({}) ===", projects.len());
    for p in &projects {
        println!("  [{}] {}", p.id, p.name);
    }

    // List tasks
    let tasks = adapter
        .list_issues(PageRequest {
            limit: Some(10),
            ..Default::default()
        })
        .await?;
    println!("\n=== Tasks ({}) ===", tasks.len());
    for t in &tasks {
        let status = t.status.as_deref().unwrap_or("unknown");
        println!("  [{}] {} ({})", t.id, t.title, status);
    }

    // Create a task
    if let Some(project) = projects.first() {
        let new_task = adapter
            .create_issue(CreateIssueRequest {
                title: "Sample task from pm-asana".into(),
                description: Some("Created via pm-asana example".into()),
                project_id: Some(project.id.clone()),
                ..Default::default()
            })
            .await?;
        println!("\nCreated task: [{}] {}", new_task.id, new_task.title);

        // Add a comment
        let comment = adapter
            .create_comment(
                &new_task.id,
                CreateCommentRequest {
                    body: "Hello from pm-asana!".into(),
                },
            )
            .await?;
        println!("Added comment: {}", comment.body);

        // Complete the task
        let updated = adapter
            .update_issue(
                &new_task.id,
                UpdateIssueRequest {
                    status: Some("completed".into()),
                    ..Default::default()
                },
            )
            .await?;
        println!("Marked as: {}", updated.status.as_deref().unwrap_or("?"));

        // Clean up
        adapter.delete_issue(&new_task.id).await?;
        println!("Deleted task {}", new_task.id);
    }

    // List tags
    let tags = adapter.list_tags(PageRequest::default()).await?;
    println!("\n=== Tags ({}) ===", tags.len());
    for t in &tags {
        println!("  [{}] {}", t.id, t.name);
    }

    // List workspaces
    let workspaces = adapter.list_teams(PageRequest::default()).await?;
    println!("\n=== Workspaces ===");
    for w in &workspaces {
        println!("  [{}] {}", w.id, w.name);
    }

    Ok(())
}
