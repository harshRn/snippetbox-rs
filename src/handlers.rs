use std::sync::Arc;

use crate::{
    AppState,
    models::snippet,
    templates::{HomeTemplate, TemplateData, ViewTemplate},
};

use askama::{Error, Template};
use axum::{
    extract::{Path, State},
    http::{HeaderMap, StatusCode, header},
    response::{Html, IntoResponse, Redirect, Response},
};

pub async fn home(State(state): State<Arc<AppState>>) -> Response {
    let mut headers = HeaderMap::new();
    headers.insert(header::SERVER, "Rust".parse().unwrap());
    headers.insert(header::CONTENT_TYPE, "html".parse().unwrap());

    let snippets = state.snippets.latest().await;
    if !snippets.is_err() {
        let view_snippets = snippets
            .unwrap()
            .into_iter()
            .map(ViewTemplate::from)
            .collect::<Vec<ViewTemplate>>();
        let home_template = HomeTemplate { view_snippets };
        match home_template.render() {
            Ok(html) => (StatusCode::OK, Html(html)).into_response(),
            Err(err) => {
                AppState::server_error(Box::new(err));
                (
                    StatusCode::INTERNAL_SERVER_ERROR,
                    "Template rendering error",
                )
                    .into_response()
            }
        }
    } else {
        let error = snippets.err().unwrap();
        let message = error.to_string();
        AppState::server_error(Box::new(error));
        (StatusCode::INTERNAL_SERVER_ERROR, format!("{:?}", message)).into_response()
    }
}

pub async fn snippet_view(
    Path(snippet_id): Path<u32>,
    State(state): State<Arc<AppState>>,
) -> Response {
    let result = state.snippets.get(&snippet_id).await;
    if !result.is_err() {
        let snippet = result.unwrap();
        let template = ViewTemplate::new(
            snippet.title,
            snippet.id,
            snippet.content,
            snippet.created,
            snippet.expires,
        );

        match template.render() {
            Ok(html) => (StatusCode::OK, Html(html)).into_response(),
            Err(err) => {
                AppState::server_error(Box::new(err));
                (
                    StatusCode::INTERNAL_SERVER_ERROR,
                    "Template rendering error",
                )
                    .into_response()
            }
        }
    } else {
        let error = result.err().unwrap();
        if let sqlx::error::Error::RowNotFound = error {
            return (StatusCode::NOT_FOUND, "snippet could not be found!").into_response();
        }
        let message = error.to_string();
        AppState::server_error(Box::new(error));
        (StatusCode::INTERNAL_SERVER_ERROR, format!("{:#?}", message)).into_response()
    }
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
