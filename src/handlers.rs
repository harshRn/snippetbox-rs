use std::sync::Arc;

use crate::{AppState, templates::HomeTemplate};

use askama::{Error, Template};
use axum::{
    extract::{Path, State},
    http::{HeaderMap, StatusCode, header},
    response::{IntoResponse, Redirect, Response},
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
            // server_error(e)
            (StatusCode::OK, headers, "e".to_string())
        }
    }
}

pub async fn snippet_view(Path(snippet_id): Path<u32>) -> String {
    tracing::info!("{snippet_id}");
    format!(
        "Display a specific snippet with a specific id : {}",
        snippet_id
    )
}

pub async fn snippet_create(State(state): State<Arc<AppState>>) -> &'static str {
    // Redirect the user to the relevant page for the snippet.
    "Display a form creating a new snippet"
}

// pub async fn snippet_create_post() -> impl IntoResponse {  // OR
pub async fn snippet_create_post(State(state): State<Arc<AppState>>) -> Response {
    let title = "Rust";
    let content = "Rust Rust Rust Rust Rust Rust Rust";
    let expires = 7;

    let result = state.snippets.insert(title, content, expires).await;
    let mut redirection_uri = "/".to_string();
    if let Err(e) = result {
        AppState::server_error(Box::new(e));
    } else {
        redirection_uri = format!("/snippet/view/{}", result.unwrap());
    }
    let mut headers = HeaderMap::new();
    headers.insert(header::LOCATION, redirection_uri.parse().unwrap());

    Redirect::to(&redirection_uri).into_response()
}
