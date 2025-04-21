use std::{collections::HashMap, sync::Arc};

use crate::{
    AppState,
    templates::{HomeTemplate, ViewTemplate},
    utils::form_validation::{CreateTemplate, SnippetData},
};

use askama::Template;
use axum::{
    extract::{Path, Request, State},
    http::{HeaderMap, StatusCode, header},
    response::{IntoResponse, Redirect, Response},
};
use tower_sessions::Session;

pub async fn home(State(state): State<Arc<AppState>>) -> Response {
    let snippets = state.snippets.latest().await;
    if !snippets.is_err() {
        let view_snippets = snippets
            .unwrap()
            .into_iter()
            .map(ViewTemplate::from)
            .collect::<Vec<ViewTemplate>>();
        let home_template = HomeTemplate { view_snippets };
        let template_render_result = home_template.render();
        AppState::render(template_render_result)
    } else {
        let error = snippets.err().unwrap();
        AppState::server_error(Box::new(error))
    }
}

pub async fn snippet_view(
    session: Session,
    Path(snippet_id): Path<u32>,
    State(state): State<Arc<AppState>>,
) -> Response {
    let result = state.snippets.get(&snippet_id).await;
    if !result.is_err() {
        let snippet = result.unwrap();
        let mut template = ViewTemplate::new(
            snippet.title,
            snippet.id,
            snippet.content,
            snippet.created,
            snippet.expires,
        );

        let flash_present: Option<String> = session.remove("flash").await.unwrap();
        if let Some(flash) = flash_present {
            if flash.len() > 0 {
                template.set_flash(flash);
            }
        }
        let template_render_result = template.render();
        AppState::render(template_render_result)
    } else {
        let error = result.err().unwrap();
        if let sqlx::error::Error::RowNotFound = error {
            return (StatusCode::NOT_FOUND, "snippet could not be found!").into_response();
        }
        AppState::server_error(Box::new(error))
    }
}

pub async fn snippet_create() -> Response {
    // Redirect the user to the relevant page for the snippet.
    let create = CreateTemplate {
        user_errors: HashMap::new(),
        title: "".to_string(),
        content: "".to_string(),
        expires: 365,
    };
    let template_render_result = create.render();
    AppState::render(template_render_result)
}

// pub async fn snippet_create_post() -> impl IntoResponse {  // OR
// #[axum::debug_handler]
pub async fn snippet_create_post(
    State(state): State<Arc<AppState>>,
    session: Session,
    snippet_data: SnippetData,
) -> Response {
    let (title, content, expires) = snippet_data.get_data();
    // default form data size = 10 MB, can be restricted like so
    // let single_byte = content
    //     .bytes()
    //     .collect::<Vec<u8>>()
    //     .first() // or drain
    //     .unwrap()
    //     .as
    //     .to_string();
    let result = state.snippets.insert(title, content, expires.into()).await;
    let mut redirection_uri = "/".to_string();
    let mut headers = HeaderMap::new();
    headers.insert(header::LOCATION, redirection_uri.parse().unwrap());
    if let Err(e) = result {
        AppState::server_error(Box::new(e));
    } else {
        redirection_uri = format!("/snippet/view/{}", result.unwrap());
    }
    session
        .insert("flash", "Snippet successfully created!")
        .await
        .unwrap();
    Redirect::to(&redirection_uri).into_response()
}
