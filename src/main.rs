mod handlers;
mod helpers;
mod models;
mod templates;
use std::sync::Arc;

use axum::http::{HeaderMap, StatusCode, header};
use axum::response::{IntoResponse, Response};
use clap::Parser;
use helpers::AppRouter;
use models::snippet::SnippetModel;
use sqlx::mysql::{MySqlConnectOptions, MySqlPool};
use sqlx::types::chrono::Utc;
use sqlx::{ConnectOptions, MySql, Pool};
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};

#[derive(Parser, Debug)]
#[command(version, about, long_about = None)]
struct Args {
    /// port
    #[arg(short, long)]
    port: u16,
}

struct AppState {
    snippets: models::snippet::SnippetModel,
}

impl AppState {
    pub fn server_error(e: Box<dyn std::error::Error>) -> (StatusCode, HeaderMap, String) {
        let mut headers = HeaderMap::new();
        headers.insert(header::SERVER, "Rust".parse().unwrap());
        headers.insert(header::CONTENT_TYPE, "text".parse().unwrap());
        (
            StatusCode::INTERNAL_SERVER_ERROR,
            headers,
            format!("Internal server error : {}", e),
        )
    }
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
    let args = Args::parse(); // command line specification of port

    // db prep with dsn
    let mut opts: MySqlConnectOptions = "mysql://root:fibo01123@0.0.0.0:3307/snippetbox"
        .parse()
        .unwrap_or_else(|e| {
            tracing::error!("problems with db connection : {}", e);
            panic!("");
        });
    opts = opts.log_statements(log::LevelFilter::Trace);
    // opts = opts.port(3307);
    let pool = open_db(opts).await;

    // app state
    let shared_state = Arc::new(AppState {
        snippets: SnippetModel::new(pool.clone()),
    });

    // init router with app state
    let app = AppRouter::new(shared_state.clone());
    let listener_res = tokio::net::TcpListener::bind(format!("0.0.0.0:{}", args.port)).await;
    let listener = match listener_res {
        Ok(l) => l,
        Err(e) => {
            tracing::error!("was not able to start server : {}", e);
            return;
        }
    };
    tracing::info!("server starting on :{}", args.port);
    axum::serve(listener, app.get_router())
        .await
        .unwrap_or_else(|e| tracing::error!("was not able to start server : {}", e));

    // gracefully_close_server_side_open_connection();
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

// #[derive(sqlx::FromRow)]
// struct Snippet {
//     id: i32,
//     title: String,
//     expires: sqlx::types::chrono::DateTime<Utc>,
// }

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
