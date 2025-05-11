use crate::handlers::{
    hn, user_login, user_login_post, user_logout_post, user_signup, user_signup_post,
};
use crate::middleware::{common_headers, request_ip, require_auth};
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
use tower_http::timeout::TimeoutLayer;
use tower_sessions::cookie::SameSite;
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
            .with_secure(false) // this should be true ???
            .with_same_site(SameSite::Lax)
            .with_expiry(Expiry::OnInactivity(Duration::hours(12)));

        // timeouts
        // https://github.com/tokio-rs/axum/blob/8762520da82cd99b78b35869069b36cfa305d4b9/axum-extra/src/middleware.rs#L15
        // https://github.com/tokio-rs/axum/blob/8762520da82cd99b78b35869069b36cfa305d4b9/examples/graceful-shutdown/src/main.rs#L13
        // https://github.com/tokio-rs/axum/blob/8762520da82cd99b78b35869069b36cfa305d4b9/axum/src/docs/middleware.md?plain=1#L58
        // let option_timeout = Some(std::time::Duration::new(10, 0));
        // let timeout_layer = option_timeout.map(TimeoutLayer::new);
        let timeout = std::time::Duration::new(10, 0);
        let tl = TimeoutLayer::new(timeout);
        let router = Router::new()
            .route_with_tsr("/snippet/create", get(snippet_create))
            .route("/snippet/create", post(snippet_create_post))
            .route("/user/logout", post(user_logout_post))
            .route_layer(axum::middleware::from_fn(require_auth)) // every route above this layer will have this middleware attached to it
            .route("/a", get(hn))
            .route("/", get(home))
            .route_with_tsr("/snippet/view/{id}", get(snippet_view))
            .route("/user/signup", get(user_signup))
            .route("/user/signup", post(user_signup_post))
            .route("/user/login", get(user_login))
            .route("/user/login", post(user_login_post))
            .nest_service("/static", ServeDir::new("static"))
            .layer(session_layer)
            .layer(axum::middleware::from_fn(common_headers))
            .layer(CatchPanicLayer::new())
            .layer(tl)
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
