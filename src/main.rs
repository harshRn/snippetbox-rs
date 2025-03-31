mod handlers;
mod templates;
use crate::handlers::{home, snippet_create, snippet_create_post, snippet_view};
// use crate::handlers;
use axum::{
    Router,
    extract::Path,
    http::{
        StatusCode, Uri,
        header::{self, HeaderMap, HeaderName},
    },
    response::IntoResponse,
    routing::{get, post},
};
// for trailing slash routes
//https://github.com/tokio-rs/axum/issues/1118
use axum_extra::routing::RouterExt;

use env_logger::Env;
use log::{Level, debug, error, info, log_enabled};

fn init_logger() {
    env_logger::Builder::from_env(Env::default().default_filter_or("info")).init();
}

#[tokio::main]
async fn main() {
    init_logger();
    // our router
    let app = Router::new()
        .route("/", get(home))
        .route_with_tsr("/snippet/view/{id}", get(snippet_view))
        .route_with_tsr("/snippet/create", get(snippet_create))
        .route("/snippet/create", post(snippet_create_post));

    let listener_res = tokio::net::TcpListener::bind("0.0.0.0:4000").await;
    let listener = match listener_res {
        Ok(l) => l,
        Err(e) => {
            error!("was not able to start server : {}", e);
            return;
        }
    };
    info!("server starting on :4000");
    axum::serve(listener, app)
        .await
        .unwrap_or_else(|e| error!("was not able to start server : {}", e));
}
