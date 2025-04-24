mod handlers;
mod models;
mod templates;
use std::net::SocketAddr;
use std::sync::Arc;
mod middleware;
mod routes;
mod utils;

use askama::Error;
use axum::http::StatusCode;
use axum::response::{Html, IntoResponse, Response};
use clap::Parser;
use models::snippet::SnippetModel;
use routes::AppRouter;
use sqlx::mysql::{MySqlConnectOptions, MySqlPool};
use sqlx::{ConnectOptions, MySql, Pool};
use tokio::{signal, task::AbortHandle};
use tower_sessions::session_store::ExpiredDeletion;
use tower_sessions_sqlx_store::MySqlStore;
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};

#[derive(Parser, Debug)]
#[command(version, about, long_about = None)]
struct Args {
    #[arg(short, long)]
    port: u16,
}

struct AppState {
    snippets: models::snippet::SnippetModel,
}

impl AppState {
    pub fn server_error(e: Box<dyn std::error::Error>) -> Response {
        (
            StatusCode::INTERNAL_SERVER_ERROR,
            format!("Internal server error : {}", e),
        )
            .into_response()
    }

    pub fn render(render_result: Result<String, Error>) -> Response {
        match render_result {
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
    }
}

#[tokio::main]
async fn main() {
    tracing_subscriber::registry()
        .with(
            tracing_subscriber::EnvFilter::try_from_default_env().unwrap_or_else(|_a| {
                // axum logs rejections from built-in extractors with the `axum::rejection`
                // target, at `TRACE` level. `axum::rejection=trace` enables showing those events
                format!(
                    "{}=debug,tower_http=debug,axum::rejection=trace,sqlx=debug,tower_sessions=debug,tower-sessions-sqlx-store=debug",
                    env!("CARGO_CRATE_NAME")
                )
                .into()
            }),
        )
        .with(tracing_subscriber::fmt::layer())
        .init();
    let args = Args::parse(); // command line specification of port

    // db prep with dsn
    let mut opts: MySqlConnectOptions = "mysql://root:fibo01123@0.0.0.0:3307/snippetbox"
        .parse()
        .unwrap_or_else(|e| {
            tracing::error!("problems with db connection : {}", e);
            panic!("");
        });
    opts = opts.log_statements(log::LevelFilter::Trace);
    let pool = open_db(opts).await;

    // session setup
    let session_store = MySqlStore::new(pool.clone())
        .with_schema_name("snippetbox")
        .unwrap()
        .with_table_name("sessions")
        .unwrap();
    session_store.migrate().await.unwrap();

    let deletion_task = tokio::task::spawn(
        session_store
            .clone()
            // run deletion once every 10 minutes
            .continuously_delete_expired(tokio::time::Duration::from_secs(600)),
    );

    // app state
    let shared_state = Arc::new(AppState {
        snippets: SnippetModel::new(pool.clone()),
    });

    // init router with app state
    let app = AppRouter::new(shared_state.clone(), session_store);
    let listener_res = tokio::net::TcpListener::bind(format!("0.0.0.0:{}", args.port)).await;
    let listener = match listener_res {
        Ok(l) => l,
        Err(e) => {
            tracing::error!("was not able to start server : {}", e);
            return;
        }
    };
    tracing::info!("server starting on :{}", args.port);
    axum::serve(
        listener,
        app.get_router()
            .into_make_service_with_connect_info::<SocketAddr>(),
    )
    .with_graceful_shutdown(shutdown_signal(deletion_task.abort_handle()))
    .await
    .unwrap_or_else(|e| tracing::error!("was not able to start server : {}", e));

    deletion_task.await.unwrap().unwrap();
    // gracefully_close_server_side_open_connection();
}

async fn shutdown_signal(deletion_task_abort_handle: AbortHandle) {
    let ctrl_c = async {
        signal::ctrl_c()
            .await
            .expect("failed to install Ctrl+C handler");
    };

    // UNIX only
    let terminate = async {
        signal::unix::signal(signal::unix::SignalKind::terminate())
            .expect("failed to install signal handler")
            .recv()
            .await;
    };

    tokio::select! {
        _ = ctrl_c => { deletion_task_abort_handle.abort() },
        _ = terminate => { deletion_task_abort_handle.abort() },
    }
}

// struct AppError(anyhow::Error);

// // Tell axum how to convert `AppError` into a response.
// impl IntoResponse for AppError {
//     fn into_response(self) -> Response {
//         (
//             StatusCode::INTERNAL_SERVER_ERROR,
//             format!("Something went wrong: {}", self.0),
//         )
//             .into_response()
//     }
// }

// This enables using `?` on functions that return `Result<_, anyhow::Error>` to turn them into
// `Result<_, AppError>`. That way you don't need to do that manually.
// impl<E> From<E> for AppError
// where
//     E: Into<anyhow::Error>,
// {
//     fn from(err: E) -> Self {
//         Self(err.into())
//     }
// }

// SERVER SIDE CONNECTION TO THE DATABASE MUST BE CLOSED BEFORE THE MAIN FUNCTION EXIT
// IN THE EVENT OF A CRASH HOW DO YOU ENSURE THAT THIS HAPPENS
//  fn gracefully_close_server_side_open_connection() {}

async fn open_db(dsn: MySqlConnectOptions) -> Pool<MySql> {
    let pool = match MySqlPool::connect_with(dsn).await {
        Ok(p) => p,
        Err(e) => {
            println!("{}", e);
            panic!("");
        }
    };
    pool
    // let rows = sqlx::query_as::<_, Snippet>("SELECT id, title, expires FROM snippets")
    //     .fetch_all(&pool)
    //     .await
    //     .unwrap();
    // for row in rows.into_iter() {
    //     println!(
    //         "id: {}\ttitle: {}\texpires: {}\n",
    //         row.id, row.title, row.expires
    //     );
    // }
    // Ok(())
}
