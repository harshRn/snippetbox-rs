use std::net::SocketAddr;

use askama::Template;
use axum::{
    extract::{ConnectInfo, Request},
    http::header::CACHE_CONTROL,
    middleware::Next,
    response::{IntoResponse, Redirect, Response},
};
use tower_sessions::Session;

use crate::{AppState, utils::login_form_validation::LoginTemplate};

pub async fn common_headers(request: Request, next: Next) -> Response {
    // any code here will be executed before the processing of the request
    //        // Any code here will execute on the way down the chain.
    // for example : authorization middleware will do it's thing here

    let mut response = next.run(request).await;

    // Any code here will execute on the way back up the chain.

    let headers = response.headers_mut();
    headers.insert(
        "Content-Security-Policy",
        "default-src 'self'; style-src 'self' fonts.googleapis.com; font-src fonts.gstatic.com"
            .parse()
            .unwrap(),
    );
    headers.insert(
        "Referrer-Policy",
        "origin-when-cross-origin".parse().unwrap(),
    );
    headers.insert("X-Content-Type-Options", "nosniff".parse().unwrap());
    headers.insert("X-Frame-Options", "deny".parse().unwrap());
    headers.insert("X-XSS-Protection", "0".parse().unwrap());
    headers.insert("Server", "Rust".parse().unwrap());
    response
}

pub async fn request_ip(
    ConnectInfo(addr): ConnectInfo<SocketAddr>,
    mut request: Request,
    next: Next,
) -> Response {
    let headers = request.headers_mut();
    headers.append(
        "user-ip",
        addr.to_string()
            .parse()
            .unwrap_or_else(|_| "unknown".parse().unwrap()),
    );
    next.run(request).await
}

pub async fn require_auth(session: Session, mut request: Request, next: Next) -> Response {
    if !AppState::is_authenticated(session).await {
        return Redirect::to("/user/login").into_response();
    }
    let headers = request.headers_mut();
    headers.append(CACHE_CONTROL, "no-store".parse().unwrap());
    next.run(request).await
}
