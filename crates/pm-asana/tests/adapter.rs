use mockito::{Matcher, Server};
use pm_asana::AsanaAdapter;
use pm_core::*;
use serde_json::json;

fn asana_response(data: serde_json::Value) -> String {
    json!({ "data": data }).to_string()
}

#[tokio::test]
async fn platform_returns_asana() {
    let adapter = AsanaAdapter::new("token");
    assert_eq!(adapter.platform(), "asana");
}

#[tokio::test]
async fn list_issues_returns_mapped_tasks() {
    let mut server = Server::new_async().await;
    let _mock = server
        .mock("GET", Matcher::Regex(r"^/tasks".to_string()))
        .with_status(200)
        .with_body(asana_response(json!([
            {
                "gid": "task_1",
                "name": "Fix login bug",
                "notes": "Login fails on Safari",
                "completed": false,
                "assignee": { "gid": "u1", "name": "Alice" },
                "memberships": [{ "project": { "gid": "p1", "name": "Core" } }],
                "tags": [{ "name": "bug" }],
                "created_at": "2026-01-01T00:00:00Z",
                "modified_at": "2026-01-02T00:00:00Z"
            }
        ])))
        .create_async()
        .await;

    let adapter = AsanaAdapter::new("token")
        .with_base_url(server.url())
        .with_workspace("ws_1");
    let issues = adapter
        .list_issues(PageRequest {
            limit: Some(10),
            ..Default::default()
        })
        .await
        .unwrap();

    assert_eq!(issues.len(), 1);
    assert_eq!(issues[0].id, "task_1");
    assert_eq!(issues[0].title, "Fix login bug");
    assert_eq!(
        issues[0].description.as_deref(),
        Some("Login fails on Safari")
    );
    assert_eq!(issues[0].status.as_deref(), Some("active"));
    assert_eq!(issues[0].assignee_name.as_deref(), Some("Alice"));
    assert_eq!(issues[0].project_name.as_deref(), Some("Core"));
    assert_eq!(issues[0].labels, vec!["bug"]);
}

#[tokio::test]
async fn get_issue_returns_task() {
    let mut server = Server::new_async().await;
    let _mock = server
        .mock("GET", Matcher::Regex(r"^/tasks/task_42".to_string()))
        .with_status(200)
        .with_body(asana_response(json!({
            "gid": "task_42",
            "name": "Design review",
            "notes": null,
            "completed": true,
            "assignee": null,
            "memberships": [],
            "tags": [],
            "created_at": "2026-02-01T00:00:00Z",
            "modified_at": "2026-02-10T00:00:00Z"
        })))
        .create_async()
        .await;

    let adapter = AsanaAdapter::new("token").with_base_url(server.url());
    let issue = adapter.get_issue("task_42").await.unwrap();

    assert_eq!(issue.id, "task_42");
    assert_eq!(issue.title, "Design review");
    assert_eq!(issue.status.as_deref(), Some("completed"));
}

#[tokio::test]
async fn create_issue_posts_task() {
    let mut server = Server::new_async().await;
    let _mock = server
        .mock("POST", Matcher::Regex(r"^/tasks".to_string()))
        .match_body(Matcher::JsonString(
            json!({ "data": { "name": "New task", "workspace": "ws_1" } }).to_string(),
        ))
        .with_status(201)
        .with_body(asana_response(json!({
            "gid": "task_new",
            "name": "New task",
            "notes": null,
            "completed": false,
            "assignee": null,
            "memberships": [],
            "tags": [],
            "created_at": "2026-03-01T00:00:00Z",
            "modified_at": "2026-03-01T00:00:00Z"
        })))
        .create_async()
        .await;

    let adapter = AsanaAdapter::new("token")
        .with_base_url(server.url())
        .with_workspace("ws_1");
    let issue = adapter
        .create_issue(CreateIssueRequest {
            title: "New task".into(),
            ..Default::default()
        })
        .await
        .unwrap();

    assert_eq!(issue.id, "task_new");
    assert_eq!(issue.title, "New task");
    assert_eq!(issue.status.as_deref(), Some("active"));
}

#[tokio::test]
async fn update_issue_puts_task() {
    let mut server = Server::new_async().await;
    let _mock = server
        .mock("PUT", Matcher::Regex(r"^/tasks/task_1".to_string()))
        .with_status(200)
        .with_body(asana_response(json!({
            "gid": "task_1",
            "name": "Updated task",
            "notes": null,
            "completed": true,
            "assignee": null,
            "memberships": [],
            "tags": [],
            "created_at": "2026-01-01T00:00:00Z",
            "modified_at": "2026-03-05T00:00:00Z"
        })))
        .create_async()
        .await;

    let adapter = AsanaAdapter::new("token").with_base_url(server.url());
    let issue = adapter
        .update_issue(
            "task_1",
            UpdateIssueRequest {
                title: Some("Updated task".into()),
                status: Some("completed".into()),
                ..Default::default()
            },
        )
        .await
        .unwrap();

    assert_eq!(issue.title, "Updated task");
    assert_eq!(issue.status.as_deref(), Some("completed"));
}

#[tokio::test]
async fn delete_issue_sends_delete() {
    let mut server = Server::new_async().await;
    let _mock = server
        .mock("DELETE", "/tasks/task_1")
        .with_status(200)
        .with_body(asana_response(json!({})))
        .create_async()
        .await;

    let adapter = AsanaAdapter::new("token").with_base_url(server.url());
    adapter.delete_issue("task_1").await.unwrap();
}

#[tokio::test]
async fn list_projects_returns_mapped() {
    let mut server = Server::new_async().await;
    let _mock = server
        .mock("GET", Matcher::Regex(r"^/projects".to_string()))
        .with_status(200)
        .with_body(asana_response(json!([
            {
                "gid": "p1",
                "name": "Platform",
                "notes": "Platform team project",
                "current_status": null,
                "created_at": "2026-01-01T00:00:00Z",
                "modified_at": "2026-01-15T00:00:00Z"
            }
        ])))
        .create_async()
        .await;

    let adapter = AsanaAdapter::new("token").with_base_url(server.url());
    let projects = adapter.list_projects(PageRequest::default()).await.unwrap();

    assert_eq!(projects.len(), 1);
    assert_eq!(projects[0].id, "p1");
    assert_eq!(projects[0].name, "Platform");
    assert_eq!(
        projects[0].description.as_deref(),
        Some("Platform team project")
    );
}

#[tokio::test]
async fn get_project_returns_project() {
    let mut server = Server::new_async().await;
    let _mock = server
        .mock("GET", Matcher::Regex(r"^/projects/p42".to_string()))
        .with_status(200)
        .with_body(asana_response(json!({
            "gid": "p42",
            "name": "Auth Service",
            "notes": null,
            "current_status": { "text": "on_track" },
            "created_at": "2026-01-01T00:00:00Z",
            "modified_at": "2026-02-01T00:00:00Z"
        })))
        .create_async()
        .await;

    let adapter = AsanaAdapter::new("token").with_base_url(server.url());
    let project = adapter.get_project("p42").await.unwrap();

    assert_eq!(project.id, "p42");
    assert_eq!(project.name, "Auth Service");
    assert_eq!(project.status.as_deref(), Some("on_track"));
}

#[tokio::test]
async fn list_teams_returns_workspaces() {
    let mut server = Server::new_async().await;
    let _mock = server
        .mock("GET", Matcher::Regex(r"^/workspaces".to_string()))
        .with_status(200)
        .with_body(asana_response(json!([
            { "gid": "ws_1", "name": "Quantum Box" }
        ])))
        .create_async()
        .await;

    let adapter = AsanaAdapter::new("token").with_base_url(server.url());
    let teams = adapter.list_teams(PageRequest::default()).await.unwrap();

    assert_eq!(teams.len(), 1);
    assert_eq!(teams[0].id, "ws_1");
    assert_eq!(teams[0].name, "Quantum Box");
}

#[tokio::test]
async fn list_comments_filters_comment_stories() {
    let mut server = Server::new_async().await;
    let _mock = server
        .mock("GET", Matcher::Regex(r"^/tasks/task_1/stories".to_string()))
        .with_status(200)
        .with_body(asana_response(json!([
            {
                "gid": "s1",
                "text": "LGTM!",
                "resource_subtype": "comment_added",
                "created_at": "2026-01-05T00:00:00Z",
                "created_by": { "name": "Bob" }
            },
            {
                "gid": "s2",
                "text": "Alice assigned to Bob",
                "resource_subtype": "assigned",
                "created_at": "2026-01-04T00:00:00Z",
                "created_by": { "name": "System" }
            }
        ])))
        .create_async()
        .await;

    let adapter = AsanaAdapter::new("token").with_base_url(server.url());
    let comments = adapter.list_comments("task_1").await.unwrap();

    assert_eq!(comments.len(), 1);
    assert_eq!(comments[0].id, "s1");
    assert_eq!(comments[0].body, "LGTM!");
    assert_eq!(comments[0].user_name.as_deref(), Some("Bob"));
}

#[tokio::test]
async fn create_comment_posts_story() {
    let mut server = Server::new_async().await;
    let _mock = server
        .mock("POST", "/tasks/task_1/stories")
        .with_status(201)
        .with_body(asana_response(json!({
            "gid": "s_new",
            "text": "Looks good!",
            "resource_subtype": "comment_added",
            "created_at": "2026-03-10T00:00:00Z",
            "created_by": { "name": "Alice" }
        })))
        .create_async()
        .await;

    let adapter = AsanaAdapter::new("token").with_base_url(server.url());
    let comment = adapter
        .create_comment(
            "task_1",
            CreateCommentRequest {
                body: "Looks good!".into(),
            },
        )
        .await
        .unwrap();

    assert_eq!(comment.id, "s_new");
    assert_eq!(comment.body, "Looks good!");
    assert_eq!(comment.user_name.as_deref(), Some("Alice"));
}

#[tokio::test]
async fn search_returns_tasks() {
    let mut server = Server::new_async().await;
    let _mock = server
        .mock(
            "GET",
            Matcher::Regex(r"^/workspaces/ws_1/tasks/search".to_string()),
        )
        .with_status(200)
        .with_body(asana_response(json!([
            { "gid": "t99", "name": "Auth refactor", "completed": false }
        ])))
        .create_async()
        .await;

    let adapter = AsanaAdapter::new("token")
        .with_base_url(server.url())
        .with_workspace("ws_1");
    let results = adapter
        .search(SearchRequest {
            query: "auth".into(),
            limit: Some(10),
        })
        .await
        .unwrap();

    assert_eq!(results.len(), 1);
    assert_eq!(results[0].id, "t99");
    assert_eq!(results[0].title, "Auth refactor");
    assert_eq!(results[0].status.as_deref(), Some("active"));
}

#[tokio::test]
async fn search_without_workspace_returns_error() {
    let adapter = AsanaAdapter::new("token");
    let err = adapter
        .search(SearchRequest {
            query: "test".into(),
            limit: None,
        })
        .await
        .unwrap_err();
    assert!(err.to_string().contains("workspace_gid"));
}

#[tokio::test]
async fn list_tags_returns_mapped() {
    let mut server = Server::new_async().await;
    let _mock = server
        .mock("GET", Matcher::Regex(r"^/tags".to_string()))
        .with_status(200)
        .with_body(asana_response(json!([
            { "gid": "tag_1", "name": "urgent" },
            { "gid": "tag_2", "name": "backend" }
        ])))
        .create_async()
        .await;

    let adapter = AsanaAdapter::new("token")
        .with_base_url(server.url())
        .with_workspace("ws_1");
    let tags = adapter.list_tags(PageRequest::default()).await.unwrap();

    assert_eq!(tags.len(), 2);
    assert_eq!(tags[0].id, "tag_1");
    assert_eq!(tags[0].name, "urgent");
}

#[tokio::test]
async fn create_tag_posts_tag() {
    let mut server = Server::new_async().await;
    let _mock = server
        .mock("POST", "/tags")
        .with_status(201)
        .with_body(asana_response(json!({
            "gid": "tag_new",
            "name": "infra"
        })))
        .create_async()
        .await;

    let adapter = AsanaAdapter::new("token")
        .with_base_url(server.url())
        .with_workspace("ws_1");
    let tag = adapter.create_tag("infra").await.unwrap();

    assert_eq!(tag.id, "tag_new");
    assert_eq!(tag.name, "infra");
}

#[tokio::test]
async fn get_tag_returns_tag() {
    let mut server = Server::new_async().await;
    let _mock = server
        .mock("GET", Matcher::Regex(r"^/tags/tag_1".to_string()))
        .with_status(200)
        .with_body(asana_response(json!({ "gid": "tag_1", "name": "urgent" })))
        .create_async()
        .await;

    let adapter = AsanaAdapter::new("token").with_base_url(server.url());
    let tag = adapter.get_tag("tag_1").await.unwrap();

    assert_eq!(tag.name, "urgent");
}

#[tokio::test]
async fn delete_tag_sends_delete() {
    let mut server = Server::new_async().await;
    let _mock = server
        .mock("DELETE", "/tags/tag_1")
        .with_status(200)
        .with_body(asana_response(json!({})))
        .create_async()
        .await;

    let adapter = AsanaAdapter::new("token").with_base_url(server.url());
    adapter.delete_tag("tag_1").await.unwrap();
}

#[tokio::test]
async fn api_error_is_propagated() {
    let mut server = Server::new_async().await;
    let _mock = server
        .mock("GET", Matcher::Any)
        .with_status(200)
        .with_body(
            json!({
                "errors": [{ "message": "Not Authorized" }]
            })
            .to_string(),
        )
        .create_async()
        .await;

    let adapter = AsanaAdapter::new("token").with_base_url(server.url());
    let err = adapter.get_issue("bad_id").await.unwrap_err();
    assert!(err.to_string().contains("Not Authorized"));
}

#[tokio::test]
async fn create_project_posts_project() {
    let mut server = Server::new_async().await;
    let _mock = server
        .mock("POST", Matcher::Regex(r"^/projects".to_string()))
        .with_status(201)
        .with_body(asana_response(json!({
            "gid": "p_new",
            "name": "New Project",
            "notes": "Description here",
            "current_status": null,
            "created_at": "2026-03-01T00:00:00Z",
            "modified_at": "2026-03-01T00:00:00Z"
        })))
        .create_async()
        .await;

    let adapter = AsanaAdapter::new("token")
        .with_base_url(server.url())
        .with_workspace("ws_1");
    let project = adapter
        .create_project("New Project", Some("Description here"))
        .await
        .unwrap();

    assert_eq!(project.id, "p_new");
    assert_eq!(project.name, "New Project");
}

#[tokio::test]
async fn delete_project_sends_delete() {
    let mut server = Server::new_async().await;
    let _mock = server
        .mock("DELETE", "/projects/p1")
        .with_status(200)
        .with_body(asana_response(json!({})))
        .create_async()
        .await;

    let adapter = AsanaAdapter::new("token").with_base_url(server.url());
    adapter.delete_project("p1").await.unwrap();
}
