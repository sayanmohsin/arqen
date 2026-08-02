use async_trait::async_trait;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ThingdObject {
    pub id: String,
    pub collection: String,
    pub data: serde_json::Value,
    pub created_at: String,
    pub updated_at: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ThingdEvent {
    pub id: String,
    pub stream: String,
    pub event_type: String,
    pub data: serde_json::Value,
    pub timestamp: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ThingdJob {
    pub id: String,
    pub queue: String,
    pub payload: serde_json::Value,
    pub state: JobState,
    pub attempts: u32,
    pub max_retries: u32,
    pub lease_expires_at: Option<String>,
    pub created_at: String,
    pub updated_at: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum JobState {
    Queued,
    Leased,
    Completed,
    Retrying,
    Dead,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ThingdLink {
    pub id: String,
    pub source_id: String,
    pub target_id: String,
    pub relation: String,
    pub created_at: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ThingdFilter {
    pub field: String,
    pub operator: FilterOperator,
    pub value: serde_json::Value,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum FilterOperator {
    Eq,
    Ne,
    Gt,
    Lt,
    Gte,
    Lte,
    Contains,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SearchOptions {
    pub limit: usize,
    pub offset: usize,
    pub filters: Vec<ThingdFilter>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SearchResults {
    pub items: Vec<ThingdObject>,
    pub total: usize,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ThingdOperation {
    Put {
        collection: String,
        id: String,
        data: serde_json::Value,
    },
    Delete {
        collection: String,
        id: String,
    },
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ThingdOperationResult {
    pub success: bool,
    pub error: Option<String>,
}

#[async_trait]
pub trait ThingdBackend: Send + Sync {
    async fn get_object(
        &self,
        collection: &str,
        id: &str,
    ) -> Result<Option<ThingdObject>, arqen_core::AppError>;
    async fn put_object(
        &self,
        collection: &str,
        id: &str,
        data: serde_json::Value,
    ) -> Result<ThingdObject, arqen_core::AppError>;
    async fn delete_object(&self, collection: &str, id: &str) -> Result<(), arqen_core::AppError>;
    async fn query_objects(
        &self,
        collection: &str,
        filter: Option<ThingdFilter>,
    ) -> Result<Vec<ThingdObject>, arqen_core::AppError>;
    async fn count_objects(&self, collection: &str) -> Result<usize, arqen_core::AppError>;

    async fn batch_write(
        &self,
        operations: Vec<ThingdOperation>,
    ) -> Result<Vec<ThingdOperationResult>, arqen_core::AppError>;

    async fn append_event(
        &self,
        stream: &str,
        event_type: &str,
        data: serde_json::Value,
    ) -> Result<ThingdEvent, arqen_core::AppError>;
    async fn read_events(
        &self,
        stream: &str,
        from: Option<String>,
        limit: usize,
    ) -> Result<Vec<ThingdEvent>, arqen_core::AppError>;

    async fn push_job(
        &self,
        queue: &str,
        payload: serde_json::Value,
        max_retries: u32,
    ) -> Result<ThingdJob, arqen_core::AppError>;
    async fn claim_job(
        &self,
        queue: &str,
        worker_id: &str,
        lease_seconds: u32,
    ) -> Result<Option<ThingdJob>, arqen_core::AppError>;
    async fn complete_job(&self, queue: &str, job_id: &str) -> Result<(), arqen_core::AppError>;
    async fn nack_job(&self, queue: &str, job_id: &str) -> Result<(), arqen_core::AppError>;
    async fn dead_letter_job(&self, queue: &str, job_id: &str) -> Result<(), arqen_core::AppError>;

    async fn search(
        &self,
        query: &str,
        options: SearchOptions,
    ) -> Result<SearchResults, arqen_core::AppError>;

    async fn create_link(
        &self,
        source_id: &str,
        target_id: &str,
        relation: &str,
    ) -> Result<ThingdLink, arqen_core::AppError>;
    async fn get_links(
        &self,
        source_id: &str,
        relation: Option<&str>,
    ) -> Result<Vec<ThingdLink>, arqen_core::AppError>;
    async fn delete_link(&self, link_id: &str) -> Result<(), arqen_core::AppError>;

    // Fixture helpers
    async fn reset(&self) -> Result<(), arqen_core::AppError>;
    async fn seed(&self) -> Result<(), arqen_core::AppError>;
}
