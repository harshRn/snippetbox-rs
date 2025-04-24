use crate::middleware::{common_headers, request_ip};
use crate::{
    AppState,
    handlers::{home, snippet_create, snippet_create_post, snippet_view},
};
use axum::{
    Router,
    extract::MatchedPath,
    http::Request,
    routing::{get, post},
};
use std::sync::Arc;
use time::Duration;
// for trailing slash routes
//https://github.com/tokio-rs/axum/issues/1118
use axum_extra::routing::RouterExt;
use tower_http::{catch_panic::CatchPanicLayer, services::ServeDir, trace::TraceLayer};
use tower_sessions::{Expiry, SessionManagerLayer}; // import Session here
use tower_sessions_sqlx_store::MySqlStore;
use tracing::info_span;

pub struct AppRouter {
    router: Router,
}

impl AppRouter {
    pub fn new(shared_state: Arc<AppState>, session_store: MySqlStore) -> Self {
        let session_layer = SessionManagerLayer::new(session_store)
            .with_secure(false)
            .with_expiry(Expiry::OnInactivity(Duration::hours(12)));

        let router = Router::new()
            .route("/", get(home))
            .route_with_tsr("/snippet/view/{id}", get(snippet_view))
            .route_with_tsr("/snippet/create", get(snippet_create))
            .route("/snippet/create", post(snippet_create_post))
            .nest_service("/static", ServeDir::new("static"))
            .layer(session_layer)
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
                    let request_ip = request.headers().get("user-ip").unwrap().to_str().unwrap();
                    let scheme = format!("{:#?}", request.version());
                    info_span!(
                        "http_request",
                        method = ?request.method(),
                        matched_path,
                        request_ip,
                        scheme
                    )
                }),
            )
            // layering happens in the opposite order of declaration so this needs to be after the logging layer
            .layer(axum::middleware::from_fn(request_ip))
            .with_state(shared_state);

        AppRouter { router }
    }

    pub fn get_router(&self) -> Router {
        self.router.clone()
    }
}
