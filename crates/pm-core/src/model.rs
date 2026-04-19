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
