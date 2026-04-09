use clap::{Parser, Subcommand, ValueEnum};
use pm_core::*;
use pm_linear::LinearAdapter;

#[derive(Parser)]
#[command(name = "pm-sdk", about = "Project management SDK CLI")]
struct Cli {
    #[arg(long, env = "PM_PLATFORM", default_value = "linear")]
    platform: Platform,

    #[arg(long, env = "PM_FORMAT", default_value = "json")]
    format: OutputFormat,

    #[command(subcommand)]
    command: Command,
}

#[derive(Clone, ValueEnum)]
enum Platform {
    Linear,
}

#[derive(Clone, ValueEnum)]
enum OutputFormat {
    Json,
    Table,
}

#[derive(Subcommand)]
enum Command {
    /// List issues
    ListIssues {
        #[arg(long, default_value = "20")]
        limit: u32,
    },
    /// Get a single issue
    GetIssue { id: String },
    /// Create an issue
    CreateIssue {
        #[arg(long)]
        title: String,
        #[arg(long)]
        description: Option<String>,
        #[arg(long)]
        project_id: Option<String>,
        #[arg(long)]
        assignee_id: Option<String>,
    },
    /// Update an issue
    UpdateIssue {
        id: String,
        #[arg(long)]
        title: Option<String>,
        #[arg(long)]
        description: Option<String>,
        #[arg(long)]
        status: Option<String>,
        #[arg(long)]
        assignee_id: Option<String>,
    },
    /// List projects
    ListProjects {
        #[arg(long, default_value = "20")]
        limit: u32,
    },
    /// Get a project
    GetProject { id: String },
    /// List teams
    ListTeams {
        #[arg(long, default_value = "20")]
        limit: u32,
    },
    /// List comments on an issue
    ListComments { issue_id: String },
    /// Add a comment to an issue
    AddComment {
        issue_id: String,
        #[arg(long)]
        body: String,
    },
    /// Search issues
    Search {
        #[arg(long)]
        query: String,
        #[arg(long, default_value = "25")]
        limit: u32,
    },
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let cli = Cli::parse();

    let adapter: Box<dyn PmAdapter> = match cli.platform {
        Platform::Linear => {
            let token = std::env::var("LINEAR_API_KEY")
                .expect("LINEAR_API_KEY required");
            Box::new(LinearAdapter::new(&token))
        }
    };

    match cli.command {
        Command::ListIssues { limit } => {
            let issues = adapter
                .list_issues(PageRequest {
                    limit: Some(limit),
                    ..Default::default()
                })
                .await?;
            print_json(&issues);
        }
        Command::GetIssue { id } => {
            let issue = adapter.get_issue(&id).await?;
            print_json(&issue);
        }
        Command::CreateIssue {
            title,
            description,
            project_id,
            assignee_id,
        } => {
            let issue = adapter
                .create_issue(CreateIssueRequest {
                    title,
                    description,
                    project_id,
                    assignee_id,
                    ..Default::default()
                })
                .await?;
            print_json(&issue);
        }
        Command::UpdateIssue {
            id,
            title,
            description,
            status,
            assignee_id,
        } => {
            let issue = adapter
                .update_issue(
                    &id,
                    UpdateIssueRequest {
                        title,
                        description,
                        status,
                        assignee_id,
                        ..Default::default()
                    },
                )
                .await?;
            print_json(&issue);
        }
        Command::ListProjects { limit } => {
            let projects = adapter
                .list_projects(PageRequest {
                    limit: Some(limit),
                    ..Default::default()
                })
                .await?;
            print_json(&projects);
        }
        Command::GetProject { id } => {
            let project = adapter.get_project(&id).await?;
            print_json(&project);
        }
        Command::ListTeams { limit } => {
            let teams = adapter
                .list_teams(PageRequest {
                    limit: Some(limit),
                    ..Default::default()
                })
                .await?;
            print_json(&teams);
        }
        Command::ListComments { issue_id } => {
            let comments =
                adapter.list_comments(&issue_id).await?;
            print_json(&comments);
        }
        Command::AddComment { issue_id, body } => {
            let comment = adapter
                .create_comment(
                    &issue_id,
                    CreateCommentRequest { body },
                )
                .await?;
            print_json(&comment);
        }
        Command::Search { query, limit } => {
            let results = adapter
                .search(SearchRequest {
                    query,
                    limit: Some(limit),
                })
                .await?;
            print_json(&results);
        }
    }

    Ok(())
}

fn print_json<T: serde::Serialize>(v: &T) {
    println!(
        "{}",
        serde_json::to_string_pretty(v).unwrap()
    );
}
