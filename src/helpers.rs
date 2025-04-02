use std::sync::Arc;

use crate::handlers::{home, snippet_create, snippet_create_post, snippet_view};
use axum::{
    Router,
    routing::{get, post},
};
use axum_extra::routing::RouterExt;
use tower_http::services::ServeDir;

pub struct AppRouter {
    router: Router,
}

impl AppRouter {
    pub fn new() -> Self {
        let router = Router::new()
            .route("/", get(home))
            .route_with_tsr("/snippet/view/{id}", get(snippet_view))
            .route_with_tsr("/snippet/create", get(snippet_create))
            .route("/snippet/create", post(snippet_create_post))
            .nest_service("/static", ServeDir::new("static"));
        AppRouter { router }
    }

    pub fn get_router(&self) -> Router {
        self.router.clone()
    }
}
