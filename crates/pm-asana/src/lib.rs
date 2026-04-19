use pm_core::*;
use reqwest::Client;
use serde_json::{json, Value};

pub struct AsanaAdapter {
    client: Client,
    token: String,
    base_url: String,
    workspace_gid: Option<String>,
}

impl AsanaAdapter {
    pub fn new(token: &str) -> Self {
        Self {
            client: Client::new(),
            token: token.to_string(),
            base_url: "https://app.asana.com/api/1.0".to_string(),
            workspace_gid: None,
        }
    }

    pub fn with_base_url(mut self, url: String) -> Self {
        self.base_url = url;
        self
    }

    pub fn with_workspace(mut self, workspace_gid: impl Into<String>) -> Self {
        self.workspace_gid = Some(workspace_gid.into());
        self
    }

    pub fn from_env() -> Self {
        let token = std::env::var("ASANA_PAT").expect("ASANA_PAT not set");
        let workspace = std::env::var("ASANA_WORKSPACE_GID").ok();
        let mut adapter = Self::new(&token);
        if let Some(w) = workspace {
            adapter = adapter.with_workspace(w);
        }
        adapter
    }

    async fn request(
        &self,
        method: reqwest::Method,
        path: &str,
        body: Option<Value>,
    ) -> PmResult<Value> {
        let url = format!("{}{}", self.base_url, path);
        let mut req = self
            .client
            .request(method, &url)
            .header("Authorization", format!("Bearer {}", self.token))
            .header("Accept", "application/json");

        if let Some(b) = body {
            req = req.header("Content-Type", "application/json").json(&b);
        }

        let resp = req.send().await?;
        let status = resp.status().as_u16();

        if status == 204 {
            return Ok(json!(null));
        }

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

        if status >= 400 {
            let msg = body["message"].as_str().unwrap_or("API error");
            return Err(PmError::Api {
                status,
                message: msg.to_string(),
            });
        }

        body.get("data")
            .cloned()
            .ok_or_else(|| PmError::Parse("No data field in response".into()))
    }

    // ── Asana-specific Tag CRUD ──────────────────────────────

    pub async fn list_tags(&self, page: PageRequest) -> PmResult<Vec<Team>> {
        let limit = page.limit.unwrap_or(50);
        let mut path = format!("/tags?limit={}&opt_fields=gid,name", limit);
        if let Some(ws) = &self.workspace_gid {
            path.push_str(&format!("&workspace={}", ws));
        }
        let data = self.request(reqwest::Method::GET, &path, None).await?;
        Ok(data
            .as_array()
            .unwrap_or(&vec![])
            .iter()
            .map(map_tag)
            .collect())
    }

    pub async fn get_tag(&self, tag_gid: &str) -> PmResult<Team> {
        let data = self
            .request(
                reqwest::Method::GET,
                &format!("/tags/{}?opt_fields=gid,name", tag_gid),
                None,
            )
            .await?;
        Ok(map_tag(&data))
    }

    pub async fn create_tag(&self, name: &str) -> PmResult<Team> {
        let workspace = self.workspace_gid.as_deref().unwrap_or_default();
        let data = self
            .request(
                reqwest::Method::POST,
                "/tags",
                Some(json!({ "data": { "name": name, "workspace": workspace } })),
            )
            .await?;
        Ok(map_tag(&data))
    }

    pub async fn delete_tag(&self, tag_gid: &str) -> PmResult<()> {
        self.request(reqwest::Method::DELETE, &format!("/tags/{}", tag_gid), None)
            .await?;
        Ok(())
    }

    // ── Asana-specific Project create/update/delete ──────────

    pub async fn create_project(&self, name: &str, notes: Option<&str>) -> PmResult<Project> {
        let workspace = self.workspace_gid.as_deref().unwrap_or_default();
        let mut obj = json!({
            "name": name,
            "workspace": workspace,
        });
        if let Some(n) = notes {
            obj["notes"] = json!(n);
        }
        let path = format!("/projects?opt_fields={}", PROJECT_FIELDS);
        let data = self
            .request(reqwest::Method::POST, &path, Some(json!({ "data": obj })))
            .await?;
        Ok(map_project(&data))
    }

    pub async fn update_project(
        &self,
        project_gid: &str,
        name: Option<&str>,
        notes: Option<&str>,
    ) -> PmResult<Project> {
        let mut obj = json!({});
        if let Some(n) = name {
            obj["name"] = json!(n);
        }
        if let Some(n) = notes {
            obj["notes"] = json!(n);
        }
        let path = format!("/projects/{}?opt_fields={}", project_gid, PROJECT_FIELDS);
        let data = self
            .request(reqwest::Method::PUT, &path, Some(json!({ "data": obj })))
            .await?;
        Ok(map_project(&data))
    }

    pub async fn delete_project(&self, project_gid: &str) -> PmResult<()> {
        self.request(
            reqwest::Method::DELETE,
            &format!("/projects/{}", project_gid),
            None,
        )
        .await?;
        Ok(())
    }
}

// ── Field sets for opt_fields ────────────────────────────────

const TASK_FIELDS: &str = "gid,name,notes,completed,assignee.gid,assignee.name,\
    memberships.project.gid,memberships.project.name,tags.name,\
    created_at,modified_at";

const PROJECT_FIELDS: &str = "gid,name,notes,current_status.text,created_at,modified_at";

const STORY_FIELDS: &str = "gid,text,resource_subtype,created_at,created_by.name";

// ── Mapping helpers ──────────────────────────────────────────

fn map_task(v: &Value) -> Issue {
    let project = &v["memberships"][0]["project"];
    let status = if v["completed"].as_bool().unwrap_or(false) {
        "completed"
    } else {
        "active"
    };
    Issue {
        id: str_field(v, "gid"),
        identifier: None,
        title: str_field(v, "name"),
        description: v["notes"].as_str().map(String::from),
        status: Some(status.to_string()),
        priority: None,
        assignee_id: v["assignee"]["gid"].as_str().map(String::from),
        assignee_name: v["assignee"]["name"].as_str().map(String::from),
        project_id: project["gid"].as_str().map(String::from),
        project_name: project["name"].as_str().map(String::from),
        labels: v["tags"]
            .as_array()
            .map(|a| {
                a.iter()
                    .filter_map(|t| t["name"].as_str().map(String::from))
                    .collect()
            })
            .unwrap_or_default(),
        created_at: v["created_at"].as_str().map(String::from),
        updated_at: v["modified_at"].as_str().map(String::from),
    }
}

fn map_project(v: &Value) -> Project {
    Project {
        id: str_field(v, "gid"),
        name: str_field(v, "name"),
        description: v["notes"].as_str().map(String::from),
        status: v["current_status"]["text"].as_str().map(String::from),
        created_at: v["created_at"].as_str().map(String::from),
        updated_at: v["modified_at"].as_str().map(String::from),
    }
}

fn map_workspace(v: &Value) -> Team {
    Team {
        id: str_field(v, "gid"),
        name: str_field(v, "name"),
        key: None,
    }
}

fn map_tag(v: &Value) -> Team {
    Team {
        id: str_field(v, "gid"),
        name: str_field(v, "name"),
        key: None,
    }
}

fn map_story(v: &Value) -> Comment {
    Comment {
        id: str_field(v, "gid"),
        body: str_field(v, "text"),
        issue_id: None,
        user_name: v["created_by"]["name"].as_str().map(String::from),
        created_at: v["created_at"].as_str().map(String::from),
    }
}

fn str_field(v: &Value, key: &str) -> String {
    v[key].as_str().unwrap_or_default().to_string()
}

// ── PmAdapter impl ───────────────────────────────────────────

#[async_trait::async_trait]
impl PmAdapter for AsanaAdapter {
    fn platform(&self) -> &str {
        "asana"
    }

    async fn list_issues(&self, page: PageRequest) -> PmResult<Vec<Issue>> {
        let limit = page.limit.unwrap_or(50);
        let mut path = format!("/tasks?limit={}&opt_fields={}", limit, TASK_FIELDS);
        if let Some(ws) = &self.workspace_gid {
            path.push_str(&format!("&workspace={}", ws));
        }
        if let Some(after) = &page.after {
            path.push_str(&format!("&offset={}", after));
        }
        let data = self.request(reqwest::Method::GET, &path, None).await?;
        Ok(data
            .as_array()
            .unwrap_or(&vec![])
            .iter()
            .map(map_task)
            .collect())
    }

    async fn get_issue(&self, id: &str) -> PmResult<Issue> {
        let data = self
            .request(
                reqwest::Method::GET,
                &format!("/tasks/{}?opt_fields={}", id, TASK_FIELDS),
                None,
            )
            .await?;
        Ok(map_task(&data))
    }

    async fn create_issue(&self, input: CreateIssueRequest) -> PmResult<Issue> {
        let mut obj = json!({ "name": input.title });
        if let Some(notes) = &input.description {
            obj["notes"] = json!(notes);
        }
        if let Some(ws) = &self.workspace_gid {
            obj["workspace"] = json!(ws);
        }
        if let Some(pid) = &input.project_id {
            obj["projects"] = json!([pid]);
        }
        if let Some(aid) = &input.assignee_id {
            obj["assignee"] = json!(aid);
        }
        let path = format!("/tasks?opt_fields={}", TASK_FIELDS);
        let data = self
            .request(reqwest::Method::POST, &path, Some(json!({ "data": obj })))
            .await?;
        Ok(map_task(&data))
    }

    async fn update_issue(&self, id: &str, input: UpdateIssueRequest) -> PmResult<Issue> {
        let mut obj = json!({});
        if let Some(t) = &input.title {
            obj["name"] = json!(t);
        }
        if let Some(d) = &input.description {
            obj["notes"] = json!(d);
        }
        if let Some(s) = &input.status {
            obj["completed"] = json!(s.to_lowercase() == "completed");
        }
        if let Some(a) = &input.assignee_id {
            obj["assignee"] = json!(a);
        }
        let path = format!("/tasks/{}?opt_fields={}", id, TASK_FIELDS);
        let data = self
            .request(reqwest::Method::PUT, &path, Some(json!({ "data": obj })))
            .await?;
        Ok(map_task(&data))
    }

    async fn delete_issue(&self, id: &str) -> PmResult<()> {
        self.request(reqwest::Method::DELETE, &format!("/tasks/{}", id), None)
            .await?;
        Ok(())
    }

    async fn list_projects(&self, page: PageRequest) -> PmResult<Vec<Project>> {
        let limit = page.limit.unwrap_or(50);
        let mut path = format!("/projects?limit={}&opt_fields={}", limit, PROJECT_FIELDS);
        if let Some(ws) = &self.workspace_gid {
            path.push_str(&format!("&workspace={}", ws));
        }
        let data = self.request(reqwest::Method::GET, &path, None).await?;
        Ok(data
            .as_array()
            .unwrap_or(&vec![])
            .iter()
            .map(map_project)
            .collect())
    }

    async fn get_project(&self, id: &str) -> PmResult<Project> {
        let data = self
            .request(
                reqwest::Method::GET,
                &format!("/projects/{}?opt_fields={}", id, PROJECT_FIELDS),
                None,
            )
            .await?;
        Ok(map_project(&data))
    }

    async fn list_teams(&self, page: PageRequest) -> PmResult<Vec<Team>> {
        let limit = page.limit.unwrap_or(50);
        let path = format!("/workspaces?limit={}&opt_fields=gid,name", limit);
        let data = self.request(reqwest::Method::GET, &path, None).await?;
        Ok(data
            .as_array()
            .unwrap_or(&vec![])
            .iter()
            .map(map_workspace)
            .collect())
    }

    async fn list_comments(&self, issue_id: &str) -> PmResult<Vec<Comment>> {
        let path = format!("/tasks/{}/stories?opt_fields={}", issue_id, STORY_FIELDS);
        let data = self.request(reqwest::Method::GET, &path, None).await?;
        Ok(data
            .as_array()
            .unwrap_or(&vec![])
            .iter()
            .filter(|v| {
                v["resource_subtype"].as_str() == Some("comment_added")
                    || v["resource_subtype"].as_str() == Some("comment")
            })
            .map(map_story)
            .collect())
    }

    async fn create_comment(
        &self,
        issue_id: &str,
        input: CreateCommentRequest,
    ) -> PmResult<Comment> {
        let path = format!("/tasks/{}/stories", issue_id);
        let data = self
            .request(
                reqwest::Method::POST,
                &path,
                Some(json!({ "data": { "text": input.body } })),
            )
            .await?;
        Ok(map_story(&data))
    }

    async fn search(&self, input: SearchRequest) -> PmResult<Vec<SearchResult>> {
        let workspace = self
            .workspace_gid
            .as_deref()
            .ok_or_else(|| PmError::UnsupportedOperation("search requires workspace_gid".into()))?;
        let limit = input.limit.unwrap_or(25);
        let encoded: String = input
            .query
            .chars()
            .map(|c| match c {
                ' ' => '+'.to_string(),
                'a'..='z' | 'A'..='Z' | '0'..='9' | '-' | '_' | '.' | '~' => c.to_string(),
                c => format!("%{:02X}", c as u32),
            })
            .collect();
        let path = format!(
            "/workspaces/{}/tasks/search?text={}&limit={}&opt_fields=gid,name,completed",
            workspace, encoded, limit
        );
        let data = self.request(reqwest::Method::GET, &path, None).await?;
        Ok(data
            .as_array()
            .unwrap_or(&vec![])
            .iter()
            .map(|v| SearchResult {
                id: str_field(v, "gid"),
                identifier: None,
                title: str_field(v, "name"),
                status: Some(if v["completed"].as_bool().unwrap_or(false) {
                    "completed".to_string()
                } else {
                    "active".to_string()
                }),
            })
            .collect())
    }
}
