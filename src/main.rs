mod handlers;
mod templates;
use crate::handlers::{home, snippet_create, snippet_create_post, snippet_view};
use axum::{
    Router,
    http::Request,
    routing::{get, get_service, post},
};
use clap::Parser;
use tower::ServiceExt;
use tower_http::{services::ServeDir, trace::TraceLayer};
// for trailing slash routes
//https://github.com/tokio-rs/axum/issues/1118
use axum_extra::routing::RouterExt;

use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};

#[derive(Parser, Debug)]
#[command(version, about, long_about = None)]
struct Args {
    /// port
    #[arg(short, long)]
    port: u16,
}

#[tokio::main]
async fn main() {
    // init_logger();
    tracing_subscriber::registry()
        .with(
            tracing_subscriber::EnvFilter::try_from_default_env().unwrap_or_else(|_| {
                format!("{}=debug,tower_http=debug", env!("CARGO_CRATE_NAME")).into()
            }),
        )
        .with(tracing_subscriber::fmt::layer())
        .init();
    let args = Args::parse();
    // our router
    let app = Router::new()
        .route("/", get(home))
        .route_with_tsr("/snippet/view/{id}", get(snippet_view))
        .route_with_tsr("/snippet/create", get(snippet_create))
        .route("/snippet/create", post(snippet_create_post))
        .nest_service("/static", ServeDir::new("static"))
        .layer(TraceLayer::new_for_http());

    let listener_res = tokio::net::TcpListener::bind(format!("0.0.0.0:{}", args.port)).await;
    let listener = match listener_res {
        Ok(l) => l,
        Err(e) => {
            tracing::error!("was not able to start server : {}", e);
            return;
        }
    };
    tracing::info!("server starting on :{}", args.port);
    axum::serve(listener, app)
        .await
        .unwrap_or_else(|e| tracing::error!("was not able to start server : {}", e));
}
