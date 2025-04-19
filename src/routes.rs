use crate::middleware::common_headers;
use std::sync::Arc;

use crate::{
    AppState,
    handlers::{home, snippet_create, snippet_create_post, snippet_view},
};
use axum::{
    Router,
    extract::MatchedPath,
    http::{Request, header::HOST},
    routing::{get, post},
};
// for trailing slash routes
//https://github.com/tokio-rs/axum/issues/1118
use axum_extra::routing::RouterExt;
use tower_http::{catch_panic::CatchPanicLayer, services::ServeDir, trace::TraceLayer};
use tracing::info_span;

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
            .layer(axum::middleware::from_fn(common_headers))
            .layer(CatchPanicLayer::new())
            .layer(
                TraceLayer::new_for_http().make_span_with(|request: &Request<_>| {
                    // Log the matched route's path (with placeholders not filled in).
                    // Use request.uri() or OriginalUri if you want the real path.
                    let matched_path = request
                        .extensions()
                        .get::<MatchedPath>()
                        .map(MatchedPath::as_str);
                    let host = request.headers().get(HOST).unwrap().to_str().unwrap();
                    let scheme = format!("{:#?}", request.version());
                    info_span!(
                        "http_request",
                        method = ?request.method(),
                        matched_path,
                        host,
                        scheme
                    )
                }),
            )
            .with_state(shared_state);

        AppRouter { router }
    }

    pub fn get_router(&self) -> Router {
        self.router.clone()
    }
}
