// Asana API からタスクを取得して表示する例
// USAGE: ASANA_ACCESS_TOKEN=xxx ASANA_WORKSPACE_GID=xxx cargo run --example fetch_asana_tasks

use pm_asana::AsanaAdapter;
use pm_core::{PageRequest, PmAdapter};

#[tokio::main]
async fn main() {
    let token = std::env::var("ASANA_ACCESS_TOKEN")
        .or_else(|_| std::env::var("ASANA_PAT"))
        .expect("ASANA_ACCESS_TOKEN or ASANA_PAT must be set");
    let workspace_gid =
        std::env::var("ASANA_WORKSPACE_GID").expect("ASANA_WORKSPACE_GID must be set");

    let adapter = AsanaAdapter::new(&token).with_workspace(workspace_gid);

    println!("Fetching tasks from Asana...\n");

    let tasks = adapter
        .list_issues(PageRequest {
            limit: Some(50),
            ..Default::default()
        })
        .await
        .unwrap();

    if tasks.is_empty() {
        println!("No tasks found.");
        return;
    }

    println!("{:<40} {:<20} {}", "Name", "Status", "Priority");
    println!("{}", "-".repeat(72));

    for task in &tasks {
        let status = task.status.as_deref().unwrap_or("-");
        let priority = task
            .priority
            .as_ref()
            .map(|p| p.to_string())
            .unwrap_or_else(|| "-".to_string());
        println!(
            "{:<40} {:<20} {}",
            truncate(&task.title, 39),
            status,
            priority
        );
    }

    println!("\nTotal: {} tasks", tasks.len());
}

fn truncate(s: &str, max_chars: usize) -> String {
    if s.chars().count() <= max_chars {
        s.to_string()
    } else {
        let mut result: String = s.chars().take(max_chars - 1).collect();
        result.push('…');
        result
    }
}
