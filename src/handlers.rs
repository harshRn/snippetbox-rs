use crate::templates::HomeTemplate;

use askama::{Error, Template};
use axum::{
    extract::Path,
    http::{HeaderMap, StatusCode, header},
    response::IntoResponse,
};

pub async fn home() -> impl IntoResponse {
    let mut headers = HeaderMap::new();
    headers.insert(header::SERVER, "Rust".parse().unwrap());
    headers.insert(header::CONTENT_TYPE, "html".parse().unwrap());
    let body = HomeTemplate {};
    match body.render() {
        Ok(b) => (StatusCode::OK, headers, b),
        Err(e) => {
            tracing::error!("could not render template home.html : {}", e);
            server_error(e)
        }
    }
}

fn server_error(e: Error) -> (StatusCode, HeaderMap, String) {
    let mut headers = HeaderMap::new();
    headers.insert(header::SERVER, "Rust".parse().unwrap());
    headers.insert(header::CONTENT_TYPE, "html".parse().unwrap());
    (
        StatusCode::INTERNAL_SERVER_ERROR,
        headers,
        format!("could not render template home.html : {}", e),
    )
}

pub async fn snippet_view(Path(snippet_id): Path<u32>) -> String {
    tracing::info!("{snippet_id}");
    format!(
        "Display a specific snippet with a specific id : {}",
        snippet_id
    )
}

pub async fn snippet_create() -> &'static str {
    "Display a form creating a new snippet"
}

// pub async fn snippet_create_post() -> impl IntoResponse {  // OR
pub async fn snippet_create_post() -> (StatusCode, &'static str) {
    (StatusCode::CREATED, "saving a new snippet")
}
