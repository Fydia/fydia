use axum::response::IntoResponse;

pub mod api;
pub mod basic;
pub mod event;
pub mod federation;

pub async fn default() -> impl IntoResponse {
    "Default. This request will be implemented soon".to_string()
}
