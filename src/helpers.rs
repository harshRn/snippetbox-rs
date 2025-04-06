use std::sync::Arc;

use crate::{
    AppState,
    handlers::{home, snippet_create, snippet_create_post, snippet_view},
};
use axum::{
    Router,
    routing::{get, post},
};
// for trailing slash routes
//https://github.com/tokio-rs/axum/issues/1118
use axum_extra::routing::RouterExt;
use tower_http::services::ServeDir;

pub struct AppRouter {
    router: Router,
}

impl AppRouter {
    pub fn new(shared_state: Arc<AppState>) -> Self {
        let router = Router::new()
            .route("/", get(home))
            .route_with_tsr("/snippet/view/{id}", get(snippet_view))
            .route_with_tsr("/snippet/create", get(snippet_create))
            .route("/snippet/create", post(snippet_create_post))
            .nest_service("/static", ServeDir::new("static"))
            .with_state(shared_state);

        AppRouter { router }
    }

    pub fn get_router(&self) -> Router {
        self.router.clone()
    }
}
