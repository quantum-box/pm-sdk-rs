use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;

// ── Issues ───────────────────────────────────────────

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Issue {
    pub id: String,
    pub identifier: Option<String>,
    pub title: String,
    pub description: Option<String>,
    pub status: Option<String>,
    pub priority: Option<IssuePriority>,
    pub assignee_id: Option<String>,
    pub assignee_name: Option<String>,
    pub project_id: Option<String>,
    pub project_name: Option<String>,
    pub labels: Vec<String>,
    pub created_at: Option<String>,
    pub updated_at: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CustomerIssueView {
    pub platform: String,
    pub identifier: String,
    pub title: String,
    pub state_name: String,
    pub state_type: String,
    pub assignee_name: Option<String>,
    pub assignee_avatar_url: Option<String>,
    pub labels: Vec<String>,
    pub description: Option<String>,
    pub updated_at: String,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct CustomerIssueFilter {
    pub customer_key: String,
    pub project_ids: Vec<String>,
    pub label_names: Vec<String>,
    pub issue_identifiers: Vec<String>,
    pub page: PageRequest,
}

impl CustomerIssueFilter {
    pub fn new(customer_key: impl Into<String>) -> Self {
        Self {
            customer_key: customer_key.into(),
            project_ids: Vec::new(),
            label_names: Vec::new(),
            issue_identifiers: Vec::new(),
            page: PageRequest::default(),
        }
    }

    pub fn with_project_ids(
        mut self,
        project_ids: impl IntoIterator<Item = impl Into<String>>,
    ) -> Self {
        self.project_ids = project_ids
            .into_iter()
            .map(Into::into)
            .filter(|id| !id.trim().is_empty())
            .collect();
        self
    }

    pub fn with_label_names(
        mut self,
        label_names: impl IntoIterator<Item = impl Into<String>>,
    ) -> Self {
        self.label_names = label_names
            .into_iter()
            .map(Into::into)
            .filter(|name| !name.trim().is_empty())
            .collect();
        self
    }

    pub fn with_issue_identifiers(
        mut self,
        issue_identifiers: impl IntoIterator<Item = impl Into<String>>,
    ) -> Self {
        self.issue_identifiers = issue_identifiers
            .into_iter()
            .map(Into::into)
            .filter(|identifier| !identifier.trim().is_empty())
            .collect();
        self
    }

    pub fn with_page(mut self, page: PageRequest) -> Self {
        self.page = page;
        self
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum IssuePriority {
    None,
    Urgent,
    High,
    Medium,
    Low,
}

impl std::fmt::Display for IssuePriority {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::None => write!(f, "None"),
            Self::Urgent => write!(f, "Urgent"),
            Self::High => write!(f, "High"),
            Self::Medium => write!(f, "Medium"),
            Self::Low => write!(f, "Low"),
        }
    }
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct CreateIssueRequest {
    pub title: String,
    pub description: Option<String>,
    pub project_id: Option<String>,
    pub assignee_id: Option<String>,
    pub priority: Option<IssuePriority>,
    pub labels: Vec<String>,
    pub properties: BTreeMap<String, serde_json::Value>,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct UpdateIssueRequest {
    pub title: Option<String>,
    pub description: Option<String>,
    pub status: Option<String>,
    pub assignee_id: Option<String>,
    pub priority: Option<IssuePriority>,
    pub labels: Option<Vec<String>>,
    pub properties: BTreeMap<String, serde_json::Value>,
}

// ── Projects ─────────────────────────────────────────

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Project {
    pub id: String,
    pub name: String,
    pub description: Option<String>,
    pub status: Option<String>,
    pub created_at: Option<String>,
    pub updated_at: Option<String>,
}

// ── Teams ────────────────────────────────────────────

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Team {
    pub id: String,
    pub name: String,
    pub key: Option<String>,
}

// ── Comments ─────────────────────────────────────────

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Comment {
    pub id: String,
    pub body: String,
    pub issue_id: Option<String>,
    pub user_name: Option<String>,
    pub created_at: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CreateCommentRequest {
    pub body: String,
}

// ── Search ───────────────────────────────────────────

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SearchRequest {
    pub query: String,
    pub limit: Option<u32>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SearchResult {
    pub id: String,
    pub identifier: Option<String>,
    pub title: String,
    pub status: Option<String>,
}

// ── Pagination ───────────────────────────────────────

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct PageRequest {
    pub limit: Option<u32>,
    pub offset: Option<u32>,
    pub after: Option<String>,
}
