mod handlers;
mod models;
mod templates;
use std::net::SocketAddr;
use std::path::PathBuf;
use std::sync::Arc;
use std::time::Duration;
mod middleware;
mod routes;
mod utils;

use askama::Error;
use axum::http::StatusCode;
use axum::response::{Html, IntoResponse, Response};
use axum_server::{Handle, tls_rustls::RustlsConfig};
use clap::Parser;
use models::snippet::SnippetModel;
use models::users::UserModel;
use routes::AppRouter;
use sqlx::mysql::{MySqlConnectOptions, MySqlPool};
use sqlx::{ConnectOptions, MySql, Pool};
use tokio::{signal, task::AbortHandle, time::sleep};
use tower_sessions::Session;
use tower_sessions::session_store::ExpiredDeletion;
use tower_sessions_sqlx_store::MySqlStore;
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};

#[derive(Parser, Debug)]
#[command(version, about, long_about = None)]
struct Args {
    #[arg(long)]
    http_port: u16,
}

#[allow(dead_code)]
#[derive(Clone, Copy)]
struct Ports {
    http: u16,
    https: u16,
}

struct AppState {
    snippets: models::snippet::SnippetModel,
    users: models::users::UserModel,
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
        // let mut header = HeaderMap::new();
        // this error where CONTENT_LENGTH is being given a non-numerical value
        // is not being caught at compile time or run-time
        // external client like curl does point towards the error
        // header.insert(CONTENT_LENGTH, "content length".parse().unwrap());
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

    pub async fn is_authenticated(session: Session) -> bool {
        let auth_res: Result<Option<i32>, tower_sessions::session::Error> =
            session.get("authenticatedUserID").await;
        if let Ok(id_opt) = auth_res {
            if let Some(_) = id_opt {
                return true;
            }
        }
        false
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
                    "{}=debug,tower_http=debug,axum::rejection=info,sqlx=error,tower_sessions=error,tower-sessions-sqlx-store=error,axum-server=debug,rustls=debug",
                    env!("CARGO_CRATE_NAME")
                )
                .into()
            }),
        )
        .with(tracing_subscriber::fmt::layer())
        .init();
    let args = Args::parse(); // command line specification of port
    let ports = Ports {
        http: args.http_port,
        https: 3000,
    };

    // optional: spawn a second server to redirect http requests to this server
    //  tokio::spawn(redirect_http_to_https(ports));

    // configure certificate and private key used by https
    let config = RustlsConfig::from_pem_file(
        PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .join("tls")
            .join("cert.pem"),
        PathBuf::from(env!("CARGO_MANIFEST_DIR"))
            .join("tls")
            .join("key.pem"),
    )
    .await
    .unwrap();

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

    // let deletion_task =
    tokio::task::spawn(
        session_store
            .clone()
            // run deletion once every 10 minutes
            .continuously_delete_expired(tokio::time::Duration::from_secs(600)),
    );

    // app state
    let shared_state = Arc::new(AppState {
        snippets: SnippetModel::new(pool.clone()),
        users: UserModel::new(pool.clone()),
    });

    // init router with app state
    let app = AppRouter::new(shared_state.clone(), session_store);

    // ----------------------------------------------------------------------
    // https server start flow : see http server start flow below
    // ----------------------------------------------------------------------
    // let handle = Handle::new(); // to be used during graceful shutdown
    // Spawn a task to gracefully shutdown server.
    // tokio::spawn(graceful_shutdown(
    //     handle.clone(),
    //     deletion_task.abort_handle(),
    // ));

    let addr = SocketAddr::from(([127, 0, 0, 1], ports.https));
    tracing::debug!("listening securely on {}", addr);
    axum_server::bind_rustls(addr, config)
        .serve(
            app.get_router()
                .into_make_service_with_connect_info::<SocketAddr>(),
        )
        .await
        .unwrap_or_else(|e| tracing::error!("was not able to start server : {}", e));

    // ----------------------------------------------------------------------
    // http server start flow
    // ----------------------------------------------------------------------
    // let listener_res = tokio::net::TcpListener::bind(format!("0.0.0.0:{}", args.http_port)).await;
    // let listener = match listener_res {
    //     Ok(l) => l,
    //     Err(e) => {
    //         tracing::error!("was not able to start server : {}", e);
    //         return;
    //     }
    // };
    // tracing::info!("server starting on :{}", args.port);
    // axum::serve(
    //     listener,
    //     app.get_router()
    //         .into_make_service_with_connect_info::<SocketAddr>(),
    // )
    // .with_graceful_shutdown(shutdown_signal(deletion_task.abort_handle()))
    // .await
    // .unwrap_or_else(|e| tracing::error!("was not able to start server : {}", e));

    // awaiting 2 futures : the serve future does not end though ??? OR does it ????
    // deletion_task.await.unwrap().unwrap();
    println!("server is shut down");
    // gracefully_close_server_side_open_connection_to_db();
}

// async fn graceful_shutdown(handle: Handle) {
//     // Signal the server to shutdown using Handle.
//     handle.graceful_shutdown(Some(std::time::Duration::from_secs(1)));
//     // possible that some session data does not get deleted ?????????????
//     // this should not be a big cause for concern because restarting the server
//     // would ultimately remove the expired sessions anyway

//     // Print alive connection count every second.
//     loop {
//         sleep(std::time::Duration::from_secs(1)).await;

//         println!("alive connections: {}", handle.connection_count());
//     }
// }

#[allow(unused)]
async fn graceful_shutdown(handle: Handle, deletion_task_abort_handle: AbortHandle) {
    // Wait 10 seconds.
    sleep(Duration::from_secs(1)).await;

    //println!("sending graceful shutdown signal");
    deletion_task_abort_handle.abort();

    // Signal the server to shutdown using Handle.
    handle.graceful_shutdown(Some(Duration::from_secs(10)));
    println!("this will not be printed");
    // println!("signal sent");
    // Print alive connection count every second.
    // loop {
    //     sleep(Duration::from_secs(1)).await;
    //     // don't log alive connections but store it somewhere for stats and analytics
    //     // println!("alive connections: {}", handle.connection_count());
    // }
}

#[allow(unused)]
async fn shutdown_signal(deletion_task_abort_handle: AbortHandle) {
    let ctrl_c = async {
        signal::ctrl_c()
            .await
            .expect("failed to install Ctrl+C handler");
    };

    #[cfg(unix)]
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
