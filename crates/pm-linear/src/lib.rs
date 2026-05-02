//! Linear GraphQL adapter implementing [`pm_core::PmAdapter`].
//!
//! # Usage
//!
//! ```no_run
//! use pm_linear::LinearAdapter;
//! use pm_core::{PmAdapter, PageRequest};
//!
//! #[tokio::main]
//! async fn main() {
//!     let adapter = LinearAdapter::new("lin_api_xxx");
//!     let issues = adapter.list_issues(PageRequest::default()).await.unwrap();
//! }
//! ```

use pm_core::*;
use reqwest::Client;
use serde_json::{json, Value};

pub use pm_core;

/// Linear API adapter. Authenticates via a Linear API token.
pub struct LinearAdapter {
    client: Client,
    token: String,
    base_url: String,
}

impl LinearAdapter {
    pub fn new(token: &str) -> Self {
        Self {
            client: Client::new(),
            token: token.to_string(),
            base_url: "https://api.linear.app".to_string(),
        }
    }

    pub fn with_base_url(mut self, url: String) -> Self {
        self.base_url = url;
        self
    }

    async fn graphql(&self, query: &str, variables: Value) -> PmResult<Value> {
        let resp = self
            .client
            .post(format!("{}/graphql", self.base_url))
            .header("Authorization", &self.token)
            .header("Content-Type", "application/json")
            .json(&json!({ "query": query, "variables": variables }))
            .send()
            .await?;

        let status = resp.status().as_u16();
        let body: Value = resp
            .json()
            .await
            .map_err(|e| PmError::Parse(e.to_string()))?;

        if let Some(errors) = body.get("errors") {
            let msg = errors[0]["message"].as_str().unwrap_or("Unknown error");
            return Err(PmError::Api {
                status,
                message: msg.to_string(),
            });
        }

        body.get("data")
            .cloned()
            .ok_or_else(|| PmError::Parse("No data in response".into()))
    }
}

fn map_issue(v: &Value) -> Issue {
    Issue {
        id: str_field(v, "id"),
        identifier: v["identifier"].as_str().map(String::from),
        title: str_field(v, "title"),
        description: v["description"].as_str().map(String::from),
        status: v["state"]["name"].as_str().map(String::from),
        priority: v["priority"].as_u64().map(map_priority),
        assignee_id: v["assignee"]["id"].as_str().map(String::from),
        assignee_name: v["assignee"]["name"].as_str().map(String::from),
        project_id: v["project"]["id"].as_str().map(String::from),
        project_name: v["project"]["name"].as_str().map(String::from),
        labels: v["labels"]["nodes"]
            .as_array()
            .map(|a| {
                a.iter()
                    .filter_map(|l| l["name"].as_str().map(String::from))
                    .collect()
            })
            .unwrap_or_default(),
        created_at: v["createdAt"].as_str().map(String::from),
        updated_at: v["updatedAt"].as_str().map(String::from),
    }
}

fn map_customer_issue(v: &Value) -> CustomerIssueView {
    let label_nodes = v["labels"]["nodes"].as_array().cloned().unwrap_or_default();
    let has_public_label = label_nodes
        .iter()
        .any(|label| label["name"].as_str() == Some("public"));
    let labels = label_nodes
        .iter()
        .filter_map(|label| label["name"].as_str())
        .filter(|name| !name.starts_with("internal:"))
        .filter(|name| !name.starts_with("customer:"))
        .map(String::from)
        .collect();

    CustomerIssueView {
        platform: "linear".to_string(),
        identifier: str_field(v, "identifier"),
        title: str_field(v, "title"),
        state_name: str_field(&v["state"], "name"),
        state_type: str_field(&v["state"], "type"),
        assignee_name: v["assignee"]["name"].as_str().map(String::from),
        assignee_avatar_url: v["assignee"]["avatarUrl"].as_str().map(String::from),
        labels,
        description: if has_public_label {
            v["description"].as_str().map(String::from)
        } else {
            None
        },
        updated_at: str_field(v, "updatedAt"),
    }
}

fn customer_issue_matches_filter(v: &Value, filter: &CustomerIssueFilter) -> bool {
    if !filter.project_ids.is_empty() {
        let project_id = v["project"]["id"].as_str().unwrap_or_default();
        if !filter.project_ids.iter().any(|id| id == project_id) {
            return false;
        }
    }

    if !filter.label_names.is_empty() {
        let label_nodes = v["labels"]["nodes"].as_array().cloned().unwrap_or_default();
        let has_allowed_label = label_nodes
            .iter()
            .filter_map(|label| label["name"].as_str())
            .any(|name| filter.label_names.iter().any(|allowed| allowed == name));
        if !has_allowed_label {
            return false;
        }
    }

    if !filter.issue_identifiers.is_empty() {
        let identifier = v["identifier"].as_str().unwrap_or_default();
        if !filter
            .issue_identifiers
            .iter()
            .any(|allowed| allowed == identifier)
        {
            return false;
        }
    }

    true
}

fn map_project(v: &Value) -> Project {
    Project {
        id: str_field(v, "id"),
        name: str_field(v, "name"),
        description: v["description"].as_str().map(String::from),
        status: v["state"].as_str().map(String::from),
        created_at: v["createdAt"].as_str().map(String::from),
        updated_at: v["updatedAt"].as_str().map(String::from),
    }
}

fn map_team(v: &Value) -> Team {
    Team {
        id: str_field(v, "id"),
        name: str_field(v, "name"),
        key: v["key"].as_str().map(String::from),
    }
}

fn map_comment(v: &Value) -> Comment {
    Comment {
        id: str_field(v, "id"),
        body: str_field(v, "body"),
        issue_id: v["issue"]["id"].as_str().map(String::from),
        user_name: v["user"]["name"].as_str().map(String::from),
        created_at: v["createdAt"].as_str().map(String::from),
    }
}

fn map_priority(p: u64) -> IssuePriority {
    match p {
        0 => IssuePriority::None,
        1 => IssuePriority::Urgent,
        2 => IssuePriority::High,
        3 => IssuePriority::Medium,
        _ => IssuePriority::Low,
    }
}

fn priority_to_int(p: &IssuePriority) -> u32 {
    match p {
        IssuePriority::None => 0,
        IssuePriority::Urgent => 1,
        IssuePriority::High => 2,
        IssuePriority::Medium => 3,
        IssuePriority::Low => 4,
    }
}

fn str_field(v: &Value, key: &str) -> String {
    v[key].as_str().unwrap_or_default().to_string()
}

fn is_valid_customer_key(key: &str) -> bool {
    if key.is_empty() || key.len() > 64 {
        return false;
    }
    let first = key.as_bytes()[0];
    if !first.is_ascii_lowercase() && !first.is_ascii_digit() {
        return false;
    }
    key.bytes()
        .all(|b| b.is_ascii_lowercase() || b.is_ascii_digit() || b == b'-' || b == b'_')
}

fn build_customer_filter(customer_key: &str) -> Value {
    json!({
        "labels": {
            "some": {
                "name": { "eq": format!("customer:{customer_key}") }
            }
        }
    })
}

const ISSUE_FIELDS: &str = "id identifier title description priority createdAt updatedAt state { name } assignee { id name } project { id name } labels { nodes { name } }";
const CUSTOMER_ISSUE_FIELDS: &str = "identifier title description updatedAt state { name type } assignee { name avatarUrl } project { id } labels { nodes { name } }";
const PROJECT_FIELDS: &str = "id name description state createdAt updatedAt";
const TEAM_FIELDS: &str = "id name key";
const COMMENT_FIELDS: &str = "id body createdAt issue { id } user { name }";

#[async_trait::async_trait]
impl PmAdapter for LinearAdapter {
    fn platform(&self) -> &str {
        "linear"
    }

    async fn list_issues(&self, page: PageRequest) -> PmResult<Vec<Issue>> {
        let limit = page.limit.unwrap_or(50);
        let q = format!(
            "query($first: Int, $after: String) {{ issues(first: $first, after: $after) {{ nodes {{ {ISSUE_FIELDS} }} }} }}"
        );
        let vars = json!({
            "first": limit,
            "after": page.after,
        });
        let data = self.graphql(&q, vars).await?;
        let nodes = data["issues"]["nodes"]
            .as_array()
            .cloned()
            .unwrap_or_default();
        Ok(nodes.iter().map(map_issue).collect())
    }

    async fn list_customer_issues(
        &self,
        filter: CustomerIssueFilter,
    ) -> PmResult<Vec<CustomerIssueView>> {
        if !is_valid_customer_key(&filter.customer_key) {
            return Ok(Vec::new());
        }

        let limit = filter.page.limit.unwrap_or(50);
        let q = format!(
            "query($first: Int, $after: String, $filter: IssueFilter!) {{ issues(first: $first, after: $after, filter: $filter, orderBy: updatedAt) {{ nodes {{ {CUSTOMER_ISSUE_FIELDS} }} }} }}"
        );
        let vars = json!({
            "first": limit,
            "after": filter.page.after,
            "filter": build_customer_filter(&filter.customer_key),
        });
        let data = self.graphql(&q, vars).await?;
        let nodes = data["issues"]["nodes"]
            .as_array()
            .cloned()
            .unwrap_or_default();
        Ok(nodes
            .iter()
            .filter(|node| customer_issue_matches_filter(node, &filter))
            .map(map_customer_issue)
            .collect())
    }

    async fn get_issue(&self, id: &str) -> PmResult<Issue> {
        let q = format!("query($id: String!) {{ issue(id: $id) {{ {ISSUE_FIELDS} }} }}");
        let data = self.graphql(&q, json!({ "id": id })).await?;
        Ok(map_issue(&data["issue"]))
    }

    async fn create_issue(&self, input: CreateIssueRequest) -> PmResult<Issue> {
        let mut vars = json!({
            "title": input.title,
        });
        let obj = vars.as_object_mut().unwrap();
        if let Some(d) = &input.description {
            obj.insert("description".into(), json!(d));
        }
        if let Some(pid) = &input.project_id {
            obj.insert("projectId".into(), json!(pid));
        }
        if let Some(aid) = &input.assignee_id {
            obj.insert("assigneeId".into(), json!(aid));
        }
        if let Some(p) = &input.priority {
            obj.insert("priority".into(), json!(priority_to_int(p)));
        }

        let q = format!(
            "mutation($input: IssueCreateInput!) {{ issueCreate(input: $input) {{ issue {{ {ISSUE_FIELDS} }} }} }}"
        );
        let data = self.graphql(&q, json!({ "input": vars })).await?;
        Ok(map_issue(&data["issueCreate"]["issue"]))
    }

    async fn update_issue(&self, id: &str, input: UpdateIssueRequest) -> PmResult<Issue> {
        let mut vars = json!({});
        let obj = vars.as_object_mut().unwrap();
        if let Some(t) = &input.title {
            obj.insert("title".into(), json!(t));
        }
        if let Some(d) = &input.description {
            obj.insert("description".into(), json!(d));
        }
        if let Some(s) = &input.status {
            obj.insert("stateId".into(), json!(s));
        }
        if let Some(a) = &input.assignee_id {
            obj.insert("assigneeId".into(), json!(a));
        }
        if let Some(p) = &input.priority {
            obj.insert("priority".into(), json!(priority_to_int(p)));
        }

        let q = format!(
            "mutation($id: String!, $input: IssueUpdateInput!) {{ issueUpdate(id: $id, input: $input) {{ issue {{ {ISSUE_FIELDS} }} }} }}"
        );
        let data = self.graphql(&q, json!({ "id": id, "input": vars })).await?;
        Ok(map_issue(&data["issueUpdate"]["issue"]))
    }

    async fn list_projects(&self, page: PageRequest) -> PmResult<Vec<Project>> {
        let limit = page.limit.unwrap_or(50);
        let q = format!(
            "query($first: Int, $after: String) {{ projects(first: $first, after: $after) {{ nodes {{ {PROJECT_FIELDS} }} }} }}"
        );
        let data = self
            .graphql(&q, json!({ "first": limit, "after": page.after }))
            .await?;
        let nodes = data["projects"]["nodes"]
            .as_array()
            .cloned()
            .unwrap_or_default();
        Ok(nodes.iter().map(map_project).collect())
    }

    async fn get_project(&self, id: &str) -> PmResult<Project> {
        let q = format!("query($id: String!) {{ project(id: $id) {{ {PROJECT_FIELDS} }} }}");
        let data = self.graphql(&q, json!({ "id": id })).await?;
        Ok(map_project(&data["project"]))
    }

    async fn list_teams(&self, page: PageRequest) -> PmResult<Vec<Team>> {
        let limit = page.limit.unwrap_or(50);
        let q = format!(
            "query($first: Int) {{ teams(first: $first) {{ nodes {{ {TEAM_FIELDS} }} }} }}"
        );
        let data = self.graphql(&q, json!({ "first": limit })).await?;
        let nodes = data["teams"]["nodes"]
            .as_array()
            .cloned()
            .unwrap_or_default();
        Ok(nodes.iter().map(map_team).collect())
    }

    async fn list_comments(&self, issue_id: &str) -> PmResult<Vec<Comment>> {
        let q = format!(
            "query($id: String!) {{ issue(id: $id) {{ comments {{ nodes {{ {COMMENT_FIELDS} }} }} }} }}"
        );
        let data = self.graphql(&q, json!({ "id": issue_id })).await?;
        let nodes = data["issue"]["comments"]["nodes"]
            .as_array()
            .cloned()
            .unwrap_or_default();
        Ok(nodes.iter().map(map_comment).collect())
    }

    async fn create_comment(
        &self,
        issue_id: &str,
        input: CreateCommentRequest,
    ) -> PmResult<Comment> {
        let q = format!(
            "mutation($input: CommentCreateInput!) {{ commentCreate(input: $input) {{ comment {{ {COMMENT_FIELDS} }} }} }}"
        );
        let data = self
            .graphql(
                &q,
                json!({ "input": { "issueId": issue_id, "body": input.body } }),
            )
            .await?;
        Ok(map_comment(&data["commentCreate"]["comment"]))
    }

    async fn search(&self, input: SearchRequest) -> PmResult<Vec<SearchResult>> {
        let limit = input.limit.unwrap_or(25);
        let q = format!(
            "query($term: String!, $first: Int) {{ searchIssues(term: $term, first: $first) {{ nodes {{ {ISSUE_FIELDS} }} }} }}"
        );
        let data = self
            .graphql(&q, json!({ "term": input.query, "first": limit }))
            .await?;
        let nodes = data["searchIssues"]["nodes"]
            .as_array()
            .cloned()
            .unwrap_or_default();
        Ok(nodes
            .iter()
            .map(|v| SearchResult {
                id: str_field(v, "id"),
                identifier: v["identifier"].as_str().map(String::from),
                title: str_field(v, "title"),
                status: v["state"]["name"].as_str().map(String::from),
            })
            .collect())
    }
}
