use mockito::{Matcher, Server};
use pm_core::*;
use pm_linear::LinearAdapter;
use serde_json::json;

fn gql_response(data: serde_json::Value) -> String {
    json!({ "data": data }).to_string()
}

#[tokio::test]
async fn platform_returns_linear() {
    let adapter = LinearAdapter::new("token");
    assert_eq!(adapter.platform(), "linear");
}

#[tokio::test]
async fn list_issues_returns_mapped_issues() {
    let mut server = Server::new_async().await;
    let _mock = server
        .mock("POST", "/graphql")
        .match_body(Matcher::Regex("issues".into()))
        .with_status(200)
        .with_body(gql_response(json!({
            "issues": {
                "nodes": [
                    {
                        "id": "iss_1",
                        "identifier": "PLT-1",
                        "title": "Fix bug",
                        "description": "desc",
                        "priority": 2,
                        "state": { "name": "In Progress" },
                        "assignee": { "id": "u1", "name": "Alice" },
                        "project": { "id": "p1", "name": "Core" },
                        "labels": { "nodes": [{ "name": "bug" }] },
                        "createdAt": "2026-01-01T00:00:00Z",
                        "updatedAt": "2026-01-02T00:00:00Z"
                    }
                ]
            }
        })))
        .create_async()
        .await;

    let adapter = LinearAdapter::new("token").with_base_url(server.url());
    let issues = adapter
        .list_issues(PageRequest {
            limit: Some(10),
            ..Default::default()
        })
        .await
        .unwrap();

    assert_eq!(issues.len(), 1);
    assert_eq!(issues[0].identifier.as_deref(), Some("PLT-1"));
    assert_eq!(issues[0].title, "Fix bug");
    assert_eq!(issues[0].status.as_deref(), Some("In Progress"));
    assert_eq!(issues[0].priority, Some(IssuePriority::High));
    assert_eq!(issues[0].labels, vec!["bug"]);
}

#[tokio::test]
async fn list_customer_issues_filters_and_masks() {
    let mut server = Server::new_async().await;
    let _mock = server
        .mock("POST", "/graphql")
        .match_body(Matcher::Regex("customer:acme".into()))
        .with_status(200)
        .with_body(gql_response(json!({
            "issues": {
                "nodes": [
                    {
                        "identifier": "PLT-123",
                        "title": "Customer-visible work",
                        "description": "This is public.",
                        "updatedAt": "2026-05-01T00:00:00Z",
                        "state": { "name": "In Progress", "type": "started" },
                        "project": { "id": "prj_customer" },
                        "assignee": {
                            "name": "Alice",
                            "avatarUrl": "https://example.com/alice.png"
                        },
                        "labels": {
                            "nodes": [
                                { "name": "public" },
                                { "name": "roadmap" },
                                { "name": "internal:ops" },
                                { "name": "customer:acme" }
                            ]
                        }
                    },
                    {
                        "identifier": "PLT-124",
                        "title": "Private notes hidden",
                        "description": "Do not expose",
                        "updatedAt": "2026-05-02T00:00:00Z",
                        "state": { "name": "Backlog", "type": "backlog" },
                        "project": { "id": "prj_customer" },
                        "assignee": null,
                        "labels": { "nodes": [{ "name": "roadmap" }] }
                    }
                ]
            }
        })))
        .create_async()
        .await;

    let adapter = LinearAdapter::new("token").with_base_url(server.url());
    let issues = adapter
        .list_customer_issues(CustomerIssueFilter::new("acme").with_page(PageRequest {
            limit: Some(10),
            ..Default::default()
        }))
        .await
        .unwrap();

    assert_eq!(issues.len(), 2);
    assert_eq!(issues[0].platform, "linear");
    assert_eq!(issues[0].identifier, "PLT-123");
    assert_eq!(issues[0].state_type, "started");
    assert_eq!(issues[0].assignee_name.as_deref(), Some("Alice"));
    assert_eq!(issues[0].labels, vec!["public", "roadmap"]);
    assert_eq!(issues[0].description.as_deref(), Some("This is public."));
    assert_eq!(issues[1].labels, vec!["roadmap"]);
    assert!(issues[1].description.is_none());
}

#[tokio::test]
async fn list_customer_issues_applies_publication_allowlists() {
    let mut server = Server::new_async().await;
    let _mock = server
        .mock("POST", "/graphql")
        .match_body(Matcher::Regex("customer:acme".into()))
        .with_status(200)
        .with_body(gql_response(json!({
            "issues": {
                "nodes": [
                    {
                        "identifier": "PLT-10",
                        "title": "Visible issue",
                        "description": "Visible",
                        "updatedAt": "2026-05-01T00:00:00Z",
                        "state": { "name": "In Progress", "type": "started" },
                        "project": { "id": "prj_allowed" },
                        "assignee": null,
                        "labels": {
                            "nodes": [
                                { "name": "public" },
                                { "name": "roadmap" },
                                { "name": "customer:acme" }
                            ]
                        }
                    },
                    {
                        "identifier": "PLT-11",
                        "title": "Wrong project",
                        "description": "Hidden",
                        "updatedAt": "2026-05-01T00:00:00Z",
                        "state": { "name": "Todo", "type": "unstarted" },
                        "project": { "id": "prj_other" },
                        "assignee": null,
                        "labels": {
                            "nodes": [
                                { "name": "public" },
                                { "name": "roadmap" },
                                { "name": "customer:acme" }
                            ]
                        }
                    },
                    {
                        "identifier": "PLT-12",
                        "title": "Wrong label",
                        "description": "Hidden",
                        "updatedAt": "2026-05-01T00:00:00Z",
                        "state": { "name": "Todo", "type": "unstarted" },
                        "project": { "id": "prj_allowed" },
                        "assignee": null,
                        "labels": {
                            "nodes": [
                                { "name": "public" },
                                { "name": "ops" },
                                { "name": "customer:acme" }
                            ]
                        }
                    },
                    {
                        "identifier": "PLT-13",
                        "title": "Wrong identifier",
                        "description": "Hidden",
                        "updatedAt": "2026-05-01T00:00:00Z",
                        "state": { "name": "Todo", "type": "unstarted" },
                        "project": { "id": "prj_allowed" },
                        "assignee": null,
                        "labels": {
                            "nodes": [
                                { "name": "public" },
                                { "name": "roadmap" },
                                { "name": "customer:acme" }
                            ]
                        }
                    }
                ]
            }
        })))
        .create_async()
        .await;

    let adapter = LinearAdapter::new("token").with_base_url(server.url());
    let issues = adapter
        .list_customer_issues(
            CustomerIssueFilter::new("acme")
                .with_project_ids(["prj_allowed"])
                .with_label_names(["roadmap"])
                .with_issue_identifiers(["PLT-10"]),
        )
        .await
        .unwrap();

    assert_eq!(issues.len(), 1);
    assert_eq!(issues[0].identifier, "PLT-10");
    assert_eq!(issues[0].labels, vec!["public", "roadmap"]);
}

#[tokio::test]
async fn list_customer_issues_rejects_invalid_key_without_request() {
    let server = Server::new_async().await;
    let adapter = LinearAdapter::new("token").with_base_url(server.url());
    let issues = adapter
        .list_customer_issues(CustomerIssueFilter::new("bad key"))
        .await
        .unwrap();

    assert!(issues.is_empty());
}

#[tokio::test]
async fn create_issue_sends_mutation() {
    let mut server = Server::new_async().await;
    let _mock = server
        .mock("POST", "/graphql")
        .match_body(Matcher::Regex("issueCreate".into()))
        .with_status(200)
        .with_body(gql_response(json!({
            "issueCreate": {
                "issue": {
                    "id": "iss_new",
                    "identifier": "PLT-99",
                    "title": "New feature",
                    "state": { "name": "Backlog" },
                    "labels": { "nodes": [] }
                }
            }
        })))
        .create_async()
        .await;

    let adapter = LinearAdapter::new("token").with_base_url(server.url());
    let issue = adapter
        .create_issue(CreateIssueRequest {
            title: "New feature".into(),
            ..Default::default()
        })
        .await
        .unwrap();

    assert_eq!(issue.id, "iss_new");
    assert_eq!(issue.title, "New feature");
}

#[tokio::test]
async fn update_issue_sends_mutation() {
    let mut server = Server::new_async().await;
    let _mock = server
        .mock("POST", "/graphql")
        .match_body(Matcher::Regex("issueUpdate".into()))
        .with_status(200)
        .with_body(gql_response(json!({
            "issueUpdate": {
                "issue": {
                    "id": "iss_1",
                    "title": "Updated title",
                    "state": { "name": "Done" },
                    "labels": { "nodes": [] }
                }
            }
        })))
        .create_async()
        .await;

    let adapter = LinearAdapter::new("token").with_base_url(server.url());
    let issue = adapter
        .update_issue(
            "iss_1",
            UpdateIssueRequest {
                title: Some("Updated title".into()),
                ..Default::default()
            },
        )
        .await
        .unwrap();

    assert_eq!(issue.title, "Updated title");
    assert_eq!(issue.status.as_deref(), Some("Done"));
}

#[tokio::test]
async fn search_returns_results() {
    let mut server = Server::new_async().await;
    let _mock = server
        .mock("POST", "/graphql")
        .match_body(Matcher::Regex("searchIssues".into()))
        .with_status(200)
        .with_body(gql_response(json!({
            "searchIssues": {
                "nodes": [
                    {
                        "id": "iss_2",
                        "identifier": "PLT-42",
                        "title": "Auth issue",
                        "state": { "name": "Open" },
                        "labels": { "nodes": [] }
                    }
                ]
            }
        })))
        .create_async()
        .await;

    let adapter = LinearAdapter::new("token").with_base_url(server.url());
    let results = adapter
        .search(SearchRequest {
            query: "auth".into(),
            limit: Some(5),
        })
        .await
        .unwrap();

    assert_eq!(results.len(), 1);
    assert_eq!(results[0].title, "Auth issue");
    assert_eq!(results[0].identifier.as_deref(), Some("PLT-42"));
}

#[tokio::test]
async fn list_projects_returns_mapped() {
    let mut server = Server::new_async().await;
    let _mock = server
        .mock("POST", "/graphql")
        .match_body(Matcher::Regex("projects".into()))
        .with_status(200)
        .with_body(gql_response(json!({
            "projects": {
                "nodes": [
                    { "id": "p1", "name": "Core", "description": "Core platform" }
                ]
            }
        })))
        .create_async()
        .await;

    let adapter = LinearAdapter::new("token").with_base_url(server.url());
    let projects = adapter.list_projects(PageRequest::default()).await.unwrap();

    assert_eq!(projects.len(), 1);
    assert_eq!(projects[0].name, "Core");
}

#[tokio::test]
async fn list_teams_returns_mapped() {
    let mut server = Server::new_async().await;
    let _mock = server
        .mock("POST", "/graphql")
        .match_body(Matcher::Regex("teams".into()))
        .with_status(200)
        .with_body(gql_response(json!({
            "teams": {
                "nodes": [
                    { "id": "t1", "name": "Engineering", "key": "ENG" }
                ]
            }
        })))
        .create_async()
        .await;

    let adapter = LinearAdapter::new("token").with_base_url(server.url());
    let teams = adapter.list_teams(PageRequest::default()).await.unwrap();

    assert_eq!(teams.len(), 1);
    assert_eq!(teams[0].key.as_deref(), Some("ENG"));
}

#[tokio::test]
async fn create_comment_sends_mutation() {
    let mut server = Server::new_async().await;
    let _mock = server
        .mock("POST", "/graphql")
        .match_body(Matcher::Regex("commentCreate".into()))
        .with_status(200)
        .with_body(gql_response(json!({
            "commentCreate": {
                "comment": {
                    "id": "c1",
                    "body": "LGTM",
                    "issue": { "id": "iss_1" },
                    "user": { "name": "Alice" },
                    "createdAt": "2026-01-01T00:00:00Z"
                }
            }
        })))
        .create_async()
        .await;

    let adapter = LinearAdapter::new("token").with_base_url(server.url());
    let comment = adapter
        .create_comment(
            "iss_1",
            CreateCommentRequest {
                body: "LGTM".into(),
            },
        )
        .await
        .unwrap();

    assert_eq!(comment.body, "LGTM");
    assert_eq!(comment.issue_id.as_deref(), Some("iss_1"));
}

#[tokio::test]
async fn api_error_is_propagated() {
    let mut server = Server::new_async().await;
    let _mock = server
        .mock("POST", "/graphql")
        .with_status(200)
        .with_body(
            json!({
                "errors": [{ "message": "Not authorized" }]
            })
            .to_string(),
        )
        .create_async()
        .await;

    let adapter = LinearAdapter::new("token").with_base_url(server.url());
    let err = adapter.get_issue("bad").await.unwrap_err();
    assert!(err.to_string().contains("Not authorized"));
}
