use pm_asana::AsanaAdapter;
use pm_core::{PageRequest, PmAdapter};

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    // Requires ASANA_PAT and ASANA_WORKSPACE_GID environment variables
    let adapter = AsanaAdapter::from_env();

    let tasks = adapter.list_issues(PageRequest::default()).await?;
    for task in &tasks {
        println!("[{}] {}", task.id, task.title);
    }
    println!("\nTotal: {} tasks", tasks.len());
    Ok(())
}
