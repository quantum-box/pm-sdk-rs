// Linear API からissueを取得してKanban形式で表示する例
// USAGE: LINEAR_API_KEY=xxx cargo run --example fetch_linear_issues

use pm_core::{PageRequest, PmAdapter};
use pm_linear::LinearAdapter;
use std::collections::BTreeMap;

fn priority_label(issue: &pm_core::Issue) -> &'static str {
    match issue.priority {
        Some(pm_core::IssuePriority::Urgent) => "🔴 Urgent",
        Some(pm_core::IssuePriority::High) => "🟠 High",
        Some(pm_core::IssuePriority::Medium) => "🟡 Medium",
        Some(pm_core::IssuePriority::Low) => "🟢 Low",
        _ => "⚪ None",
    }
}

fn kanban_status(issue: &pm_core::Issue) -> &str {
    issue.status.as_deref().unwrap_or("Unknown")
}

#[tokio::main]
async fn main() {
    let api_key = std::env::var("LINEAR_API_KEY").expect("LINEAR_API_KEY must be set");

    let adapter = LinearAdapter::new(&api_key);

    println!("Fetching issues from Linear...\n");

    let issues = adapter
        .list_issues(PageRequest {
            limit: Some(50),
            ..Default::default()
        })
        .await
        .unwrap();

    // Group by Kanban status
    let mut by_status: BTreeMap<String, Vec<&pm_core::Issue>> = BTreeMap::new();
    for issue in &issues {
        by_status
            .entry(kanban_status(issue).to_string())
            .or_default()
            .push(issue);
    }

    for (status, group) in &by_status {
        println!("── {} ({}) ──", status, group.len());
        for issue in group {
            let id = issue.identifier.as_deref().unwrap_or(&issue.id);
            println!("  [{id}] {} | {}", issue.title, priority_label(issue));
        }
        println!();
    }

    println!("Total: {} issues", issues.len());
}
