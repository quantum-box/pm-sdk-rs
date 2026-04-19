use pm_core::{PageRequest, PmAdapter};
use pm_linear::LinearAdapter;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let api_key = std::env::var("LINEAR_API_KEY").expect("LINEAR_API_KEY must be set");
    let adapter = LinearAdapter::new(&api_key);

    let issues = adapter.list_issues(PageRequest::default()).await?;
    for issue in &issues {
        println!("[{}] {}", issue.id, issue.title);
    }
    println!("\nTotal: {} issues", issues.len());
    Ok(())
}
