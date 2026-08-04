use std::net::SocketAddr;
use std::sync::Arc;

use axum::extract::{Path, Query, State};
use axum::http::StatusCode;
use axum::response::IntoResponse;
use axum::routing::{get, post, put};
use axum::{Json, Router};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

// ============================================================================
// Domain Types
// ============================================================================

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Title {
    pub id: String,
    pub name: String,
    pub year: Option<u32>,
    pub genre: Option<String>,
    pub description: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TitleAvailability {
    pub title_id: String,
    pub regions: Vec<String>,
    pub streaming: bool,
    pub rental: bool,
    pub purchase: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LibraryItem {
    pub id: String,
    pub title_id: String,
    pub added_at: String,
    pub status: LibraryStatus,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum LibraryStatus {
    WantToWatch,
    Watching,
    Completed,
    Dropped,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Progress {
    pub episode_id: String,
    pub title_id: String,
    pub episode_number: u32,
    pub progress_seconds: u32,
    pub duration_seconds: u32,
    pub completed: bool,
    pub updated_at: String,
}

// ============================================================================
// Request Types
// ============================================================================

#[derive(Debug, Deserialize)]
pub struct SearchQuery {
    pub q: Option<String>,
    pub genre: Option<String>,
    pub limit: Option<u32>,
}

#[derive(Debug, Deserialize)]
pub struct CreateLibraryItem {
    pub title_id: String,
}

#[derive(Debug, Deserialize)]
pub struct UpdateProgress {
    pub progress_seconds: u32,
    pub duration_seconds: u32,
}

// ============================================================================
// Response Types
// ============================================================================

#[derive(Debug, Serialize)]
pub struct ApiResponse<T> {
    pub data: T,
}

#[derive(Debug, Serialize)]
pub struct ErrorResponse {
    pub error: String,
    pub code: String,
}

// ============================================================================
// Storage Trait
// ============================================================================

#[async_trait::async_trait]
pub trait Storage: Send + Sync {
    async fn search_titles(&self, query: &str, genre: Option<&str>, limit: u32) -> Result<Vec<Title>, String>;
    async fn get_title(&self, id: &str) -> Result<Option<Title>, String>;
    async fn get_title_availability(&self, title_id: &str) -> Result<Option<TitleAvailability>, String>;
    async fn list_library(&self) -> Result<Vec<LibraryItem>, String>;
    async fn add_to_library(&self, title_id: &str) -> Result<LibraryItem, String>;
    async fn update_progress(&self, episode_id: &str, progress_seconds: u32, duration_seconds: u32) -> Result<Progress, String>;
}

// ============================================================================
// In-Memory Storage
// ============================================================================

pub struct InMemoryStorage {
    titles: Vec<Title>,
    library: Vec<LibraryItem>,
    progress: Vec<Progress>,
}

impl InMemoryStorage {
    pub fn new() -> Self {
        Self {
            titles: vec![
                Title {
                    id: "title-1".to_string(),
                    name: "The Matrix".to_string(),
                    year: Some(1999),
                    genre: Some("Sci-Fi".to_string()),
                    description: Some("A computer hacker learns about the true nature of reality.".to_string()),
                },
                Title {
                    id: "title-2".to_string(),
                    name: "Inception".to_string(),
                    year: Some(2010),
                    genre: Some("Sci-Fi".to_string()),
                    description: Some("A thief who steals corporate secrets through dream-sharing technology.".to_string()),
                },
                Title {
                    id: "title-3".to_string(),
                    name: "The Dark Knight".to_string(),
                    year: Some(2008),
                    genre: Some("Action".to_string()),
                    description: Some("Batman raises the stakes in his war on crime.".to_string()),
                },
            ],
            library: Vec::new(),
            progress: Vec::new(),
        }
    }
}

#[async_trait::async_trait]
impl Storage for InMemoryStorage {
    async fn search_titles(&self, query: &str, genre: Option<&str>, limit: u32) -> Result<Vec<Title>, String> {
        let mut results: Vec<Title> = self.titles.iter()
            .filter(|t| {
                let matches_query = query.is_empty() || t.name.to_lowercase().contains(&query.to_lowercase());
                let matches_genre = genre.is_none() || t.genre.as_deref() == genre;
                matches_query && matches_genre
            })
            .cloned()
            .take(limit as usize)
            .collect();
        Ok(results)
    }

    async fn get_title(&self, id: &str) -> Result<Option<Title>, String> {
        Ok(self.titles.iter().find(|t| t.id == id).cloned())
    }

    async fn get_title_availability(&self, title_id: &str) -> Result<Option<TitleAvailability>, String> {
        if self.titles.iter().any(|t| t.id == title_id) {
            Ok(Some(TitleAvailability {
                title_id: title_id.to_string(),
                regions: vec!["US".to_string(), "UK".to_string()],
                streaming: true,
                rental: true,
                purchase: true,
            }))
        } else {
            Ok(None)
        }
    }

    async fn list_library(&self) -> Result<Vec<LibraryItem>, String> {
        Ok(self.library.clone())
    }

    async fn add_to_library(&self, title_id: &str) -> Result<LibraryItem, String> {
        if !self.titles.iter().any(|t| t.id == title_id) {
            return Err("title not found".to_string());
        }
        let item = LibraryItem {
            id: Uuid::new_v4().to_string(),
            title_id: title_id.to_string(),
            added_at: chrono::Utc::now().to_rfc3339(),
            status: LibraryStatus::WantToWatch,
        };
        Ok(item)
    }

    async fn update_progress(&self, episode_id: &str, progress_seconds: u32, duration_seconds: u32) -> Result<Progress, String> {
        let progress = Progress {
            episode_id: episode_id.to_string(),
            title_id: "title-1".to_string(),
            episode_number: 1,
            progress_seconds,
            duration_seconds,
            completed: progress_seconds >= duration_seconds,
            updated_at: chrono::Utc::now().to_rfc3339(),
        };
        Ok(progress)
    }
}

// ============================================================================
// Handlers
// ============================================================================

pub async fn health() -> impl IntoResponse {
    Json(serde_json::json!({"status": "ok"}))
}

pub async fn readiness() -> impl IntoResponse {
    Json(serde_json::json!({"status": "ok"}))
}

pub async fn search_titles(
    State(state): State<Arc<dyn Storage>>,
    Query(query): Query<SearchQuery>,
) -> Result<Json<ApiResponse<Vec<Title>>>, (StatusCode, Json<ErrorResponse>)> {
    let limit = query.limit.unwrap_or(10);
    let titles = state.search_titles(&query.q.unwrap_or_default(), query.genre.as_deref(), limit)
        .await
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, Json(ErrorResponse { error: e, code: "INTERNAL_ERROR".to_string() })))?;
    Ok(Json(ApiResponse { data: titles }))
}

pub async fn get_title(
    State(state): State<Arc<dyn Storage>>,
    Path(id): Path<String>,
) -> Result<Json<ApiResponse<Title>>, (StatusCode, Json<ErrorResponse>)> {
    state.get_title(&id)
        .await
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, Json(ErrorResponse { error: e, code: "INTERNAL_ERROR".to_string() })))?
        .map(|title| Json(ApiResponse { data: title }))
        .ok_or_else(|| (StatusCode::NOT_FOUND, Json(ErrorResponse { error: "title not found".to_string(), code: "NOT_FOUND".to_string() })))
}

pub async fn get_title_availability(
    State(state): State<Arc<dyn Storage>>,
    Path(id): Path<String>,
) -> Result<Json<ApiResponse<TitleAvailability>>, (StatusCode, Json<ErrorResponse>)> {
    state.get_title_availability(&id)
        .await
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, Json(ErrorResponse { error: e, code: "INTERNAL_ERROR".to_string() })))?
        .map(|avail| Json(ApiResponse { data: avail }))
        .ok_or_else(|| (StatusCode::NOT_FOUND, Json(ErrorResponse { error: "title not found".to_string(), code: "NOT_FOUND".to_string() })))
}

pub async fn list_library(
    State(state): State<Arc<dyn Storage>>,
) -> Result<Json<ApiResponse<Vec<LibraryItem>>>, (StatusCode, Json<ErrorResponse>)> {
    let items = state.list_library()
        .await
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, Json(ErrorResponse { error: e, code: "INTERNAL_ERROR".to_string() })))?;
    Ok(Json(ApiResponse { data: items }))
}

pub async fn add_to_library(
    State(state): State<Arc<dyn Storage>>,
    Json(input): Json<CreateLibraryItem>,
) -> Result<(StatusCode, Json<ApiResponse<LibraryItem>>), (StatusCode, Json<ErrorResponse>)> {
    let item = state.add_to_library(&input.title_id)
        .await
        .map_err(|e| (StatusCode::BAD_REQUEST, Json(ErrorResponse { error: e, code: "BAD_REQUEST".to_string() })))?;
    Ok((StatusCode::CREATED, Json(ApiResponse { data: item })))
}

pub async fn update_progress(
    State(state): State<Arc<dyn Storage>>,
    Path(episode_id): Path<String>,
    Json(input): Json<UpdateProgress>,
) -> Result<Json<ApiResponse<Progress>>, (StatusCode, Json<ErrorResponse>)> {
    let progress = state.update_progress(&episode_id, input.progress_seconds, input.duration_seconds)
        .await
        .map_err(|e| (StatusCode::INTERNAL_SERVER_ERROR, Json(ErrorResponse { error: e, code: "INTERNAL_ERROR".to_string() })))?;
    Ok(Json(ApiResponse { data: progress }))
}

// ============================================================================
// Router
// ============================================================================

pub fn create_router(storage: Arc<dyn Storage>) -> Router {
    Router::new()
        .route("/health", get(health))
        .route("/ready", get(readiness))
        .route("/v1/titles/search", get(search_titles))
        .route("/v1/titles/{id}", get(get_title))
        .route("/v1/titles/{id}/availability", get(get_title_availability))
        .route("/v1/library", get(list_library).post(add_to_library))
        .route("/v1/progress/{episode_id}", put(update_progress))
        .with_state(storage)
}

// ============================================================================
// Main
// ============================================================================

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    tracing_subscriber::fmt()
        .with_env_filter(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| "watchloom=info".into()),
        )
        .init();

    let storage: Arc<dyn Storage> = Arc::new(InMemoryStorage::new());
    let app = create_router(storage);

    let addr = SocketAddr::from(([0, 0, 0, 0], 3001));
    tracing::info!("Watchloom listening on {}", addr);

    let listener = tokio::net::TcpListener::bind(addr).await?;
    axum::serve(listener, app).await?;

    Ok(())
}

// ============================================================================
// Tests
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    fn test_storage() -> Arc<dyn Storage> {
        Arc::new(InMemoryStorage::new())
    }

    #[tokio::test]
    async fn test_health() {
        let storage = test_storage();
        let app = create_router(storage);
        let response = axum::test_helpers::test_get(&app, "/health").await;
        assert_eq!(response.status(), StatusCode::OK);
    }

    #[tokio::test]
    async fn test_readiness() {
        let storage = test_storage();
        let app = create_router(storage);
        let response = axum::test_helpers::test_get(&app, "/ready").await;
        assert_eq!(response.status(), StatusCode::OK);
    }

    #[tokio::test]
    async fn test_search_titles() {
        let storage = test_storage();
        let app = create_router(storage);
        let response = axum::test_helpers::test_get(&app, "/v1/titles/search?q=matrix").await;
        assert_eq!(response.status(), StatusCode::OK);
        let body: ApiResponse<Vec<Title>> = axum::test_helpers::test_response_body(response).await;
        assert_eq!(body.data.len(), 1);
        assert_eq!(body.data[0].name, "The Matrix");
    }

    #[tokio::test]
    async fn test_get_title() {
        let storage = test_storage();
        let app = create_router(storage);
        let response = axum::test_helpers::test_get(&app, "/v1/titles/title-1").await;
        assert_eq!(response.status(), StatusCode::OK);
        let body: ApiResponse<Title> = axum::test_helpers::test_response_body(response).await;
        assert_eq!(body.data.name, "The Matrix");
    }

    #[tokio::test]
    async fn test_get_title_not_found() {
        let storage = test_storage();
        let app = create_router(storage);
        let response = axum::test_helpers::test_get(&app, "/v1/titles/nonexistent").await;
        assert_eq!(response.status(), StatusCode::NOT_FOUND);
    }

    #[tokio::test]
    async fn test_get_title_availability() {
        let storage = test_storage();
        let app = create_router(storage);
        let response = axum::test_helpers::test_get(&app, "/v1/titles/title-1/availability").await;
        assert_eq!(response.status(), StatusCode::OK);
        let body: ApiResponse<TitleAvailability> = axum::test_helpers::test_response_body(response).await;
        assert_eq!(body.data.title_id, "title-1");
        assert!(body.data.streaming);
    }

    #[tokio::test]
    async fn test_list_library() {
        let storage = test_storage();
        let app = create_router(storage);
        let response = axum::test_helpers::test_get(&app, "/v1/library").await;
        assert_eq!(response.status(), StatusCode::OK);
        let body: ApiResponse<Vec<LibraryItem>> = axum::test_helpers::test_response_body(response).await;
        assert!(body.data.is_empty());
    }

    #[tokio::test]
    async fn test_add_to_library() {
        let storage = test_storage();
        let app = create_router(storage);
        let body = serde_json::json!({"title_id": "title-1"});
        let response = axum::test_helpers::test_post(&app, "/v1/library", body).await;
        assert_eq!(response.status(), StatusCode::CREATED);
        let body: ApiResponse<LibraryItem> = axum::test_helpers::test_response_body(response).await;
        assert_eq!(body.data.title_id, "title-1");
    }

    #[tokio::test]
    async fn test_update_progress() {
        let storage = test_storage();
        let app = create_router(storage);
        let body = serde_json::json!({"progress_seconds": 1200, "duration_seconds": 3600});
        let response = axum::test_helpers::test_put(&app, "/v1/progress/ep-1", body).await;
        assert_eq!(response.status(), StatusCode::OK);
        let body: ApiResponse<Progress> = axum::test_helpers::test_response_body(response).await;
        assert_eq!(body.data.episode_id, "ep-1");
        assert_eq!(body.data.progress_seconds, 1200);
        assert!(!body.data.completed);
    }
}
