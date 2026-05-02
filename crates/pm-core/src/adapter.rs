use crate::error::PmResult;
use crate::model::*;

/// Platform-agnostic project management adapter.
///
/// Implementations exist for Linear (GraphQL) and can be
/// extended to Jira, Asana, etc.
#[async_trait::async_trait]
pub trait PmAdapter: Send + Sync {
    /// Return the platform identifier (e.g. "linear").
    fn platform(&self) -> &str;

    // ── Issues ──────────────────────────────────────

    async fn list_issues(&self, page: PageRequest) -> PmResult<Vec<Issue>>;

    async fn list_customer_issues(
        &self,
        filter: CustomerIssueFilter,
    ) -> PmResult<Vec<CustomerIssueView>> {
        let _ = filter;
        Err(crate::PmError::UnsupportedOperation(
            "list_customer_issues".into(),
        ))
    }

    async fn get_issue(&self, id: &str) -> PmResult<Issue>;

    async fn create_issue(&self, input: CreateIssueRequest) -> PmResult<Issue>;

    async fn update_issue(&self, id: &str, input: UpdateIssueRequest) -> PmResult<Issue>;

    async fn delete_issue(&self, id: &str) -> PmResult<()> {
        let _ = id;
        Err(crate::PmError::UnsupportedOperation("delete_issue".into()))
    }

    // ── Projects ────────────────────────────────────

    async fn list_projects(&self, page: PageRequest) -> PmResult<Vec<Project>>;

    async fn get_project(&self, id: &str) -> PmResult<Project>;

    // ── Teams ───────────────────────────────────────

    async fn list_teams(&self, page: PageRequest) -> PmResult<Vec<Team>>;

    // ── Comments ────────────────────────────────────

    async fn list_comments(&self, issue_id: &str) -> PmResult<Vec<Comment>>;

    async fn create_comment(
        &self,
        issue_id: &str,
        input: CreateCommentRequest,
    ) -> PmResult<Comment>;

    // ── Search ──────────────────────────────────────

    async fn search(&self, input: SearchRequest) -> PmResult<Vec<SearchResult>>;
}
