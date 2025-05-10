use std::{collections::HashMap, sync::Arc};

use crate::{
    AppState,
    templates::{HomeTemplate, ViewTemplate},
    utils::{
        form_validation::{CreateTemplate, SnippetData},
        login_form_validation::{LoginData, LoginTemplate},
        signup_form_validation::{SignupData, SignupTemplate},
    },
};

use askama::Template;
use axum::{
    extract::{Path, State},
    http::{HeaderMap, StatusCode, header},
    response::{IntoResponse, Redirect, Response},
};
use sqlx::{error::DatabaseError, mysql::MySqlDatabaseError};
use tower_sessions::Session;

pub async fn hn() -> Response {
    (StatusCode::OK, "hello").into_response()
}

pub async fn home(State(state): State<Arc<AppState>>, session: Session) -> Response {
    let snippets = state.snippets.latest().await;
    if !snippets.is_err() {
        let view_snippets = snippets
            .unwrap()
            .into_iter()
            .map(ViewTemplate::from)
            .collect::<Vec<ViewTemplate>>();
        let flash_present: Option<String> = session.remove("flash").await.unwrap();
        let mut f = "".to_string();
        if let Some(flash) = flash_present {
            if flash.len() > 0 {
                f = flash;
            }
        }
        let home_template = HomeTemplate {
            view_snippets,
            flash: f,
        };
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
    // default form data size = 10 MB, can be restricted like so
    // let single_byte = content
    //     .bytes()
    //     .collect::<Vec<u8>>()
    //     .first() // or drain
    //     .unwrap()
    //     .as
    //     .to_string();
    let result = state
        .snippets
        .insert(
            snippet_data.title,
            snippet_data.content,
            snippet_data.expires.into(),
        )
        .await;
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

pub async fn user_signup() -> Response {
    let user = SignupTemplate {
        user_errors: HashMap::new(),
        name: "".to_string(),
        email: "".to_string(),
        password: "".to_string(),
    };

    let template_render_result = user.render();
    AppState::render(template_render_result)
}

pub async fn user_signup_post(
    State(state): State<Arc<AppState>>,
    session: Session,
    signup_data: SignupData,
) -> Response {
    let result = state
        .users
        .insert(
            signup_data.name.clone(),
            signup_data.email.clone(),
            signup_data.password.clone(),
        )
        .await;
    let mut redirection_uri = "/user/signup".to_string();
    let mut headers = HeaderMap::new();
    headers.insert(header::LOCATION, redirection_uri.parse().unwrap());

    //  centralised DB error handling required. This is too ugly
    if let Err(e) = result {
        let e: sqlx::error::Error = *(e.downcast().unwrap()); // problematic if bcrypt has an error. Fix.
        if let sqlx::error::Error::Database(err) = e {
            let x = *(err.downcast::<MySqlDatabaseError>());
            if x.number() == 1062 {
                let mut user = SignupTemplate {
                    user_errors: HashMap::new(),
                    name: signup_data.name,
                    email: signup_data.email,
                    password: "".to_string(),
                };
                user.user_errors.insert(
                    "signup_error".to_string(),
                    "This email is already in use".to_string(),
                );
                let template_render_result = user.render();
                return AppState::render(template_render_result);
            }
        }
    } else {
        redirection_uri = format!("/user/login")
    }
    session
        .insert("flash", "Your signup was successful. Please log in.")
        .await
        .unwrap();
    Redirect::to(&redirection_uri).into_response()
}

pub async fn user_login(session: Session) -> Response {
    let mut login_template = LoginTemplate::new("".to_string(), "".to_string());
    let flash_present: Option<String> = session.remove("flash").await.unwrap();
    if let Some(flash) = flash_present {
        if flash.len() > 0 {
            login_template.flash = flash;
        }
    }
    let template_render_result = login_template.render();
    AppState::render(template_render_result)
}

pub async fn user_login_post(
    State(state): State<Arc<AppState>>,
    session: Session,
    login_data: LoginData,
) -> Response {
    let redirection_uri = "/snippet/create";
    match state
        .users
        .authenticate(&login_data.email, &login_data.password)
        .await
    {
        Ok(id) => {
            tracing::info!("login successful : {}", id);
            match session.cycle_id().await {
                Ok(_) => {
                    let ses_auth_ins = session.insert("authenticatedUserID", id).await;
                    if let Err(e) = ses_auth_ins {
                        tracing::error!(
                            "could not insert the authenticated user id into the session : {}",
                            e.to_string()
                        );
                    }
                    Redirect::to(&redirection_uri).into_response()
                }
                Err(e) => {
                    tracing::info!("session could not be renewed : {}", e);
                    AppState::server_error(Box::new(e))
                }
            }
        }
        Err(e) => {
            // centralised error handling that takes errors,
            // downcasts them with proper handling and then returns the correct error type
            let mut login_template = LoginTemplate::new(login_data.email, "".to_string());
            login_template
                .user_errors
                .insert("login_error".to_string(), e.to_string());
            let template_render_result = login_template.render();
            AppState::render(template_render_result)
            // StatusUnprocessableEntity when login fails
        }
    }
}

pub async fn user_logout_post(session: Session) -> Response {
    match session.cycle_id().await {
        Ok(_) => {
            // why renew token first and then remove authenticatedUserID , why not do it before ?
            let auth_rmv: Result<Option<i32>, tower_sessions::session::Error> =
                session.remove("authenticatedUserID").await;
            match auth_rmv {
                Ok(_) => {
                    // error handling here
                    tracing::info!("session renewal successful");
                    let _ = session
                        .insert("flash", "You've been successfully logged out")
                        .await;
                    Redirect::to("/").into_response()
                }
                Err(e) => {
                    tracing::error!("problems in removing auth details from session : {}", e);
                    AppState::server_error(Box::new(e)).into_response()
                }
            }
        }
        Err(e) => {
            tracing::error!("session could not be renewed : {}", e);
            AppState::server_error(Box::new(e)).into_response()
        }
    }
}
