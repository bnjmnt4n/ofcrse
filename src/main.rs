use std::net::{IpAddr, Ipv4Addr, SocketAddr};

use axum::{
    body::Body,
    extract::Host,
    http::{Request, StatusCode},
    response::IntoResponse,
    routing::{any, get, get_service},
    Router,
};
use tower::util::ServiceExt;
use tower_http::services::{ServeDir, ServeFile};

const DEFAULT_PORT: i32 = 3000;

#[tokio::main]
async fn main() {
    let port: u16 = std::env::var("PORT")
        .unwrap_or(DEFAULT_PORT.to_string())
        .parse()
        .unwrap();
    let address = SocketAddr::new(IpAddr::V4(Ipv4Addr::new(0, 0, 0, 0)), port);

    let app = Router::new()
        .route("/", any(handler))
        .route("/*path", any(handler));

    axum::Server::bind(&address)
        .serve(app.into_make_service())
        .await
        .unwrap();
}

async fn handler(Host(hostname): Host, request: Request<Body>) -> impl IntoResponse {
    let file_server = ServeDir::new("dist").not_found_service(ServeFile::new("dist/404.html"));
    let primary_app =
        Router::new().fallback_service(get_service(file_server).handle_error(handle_error));

    let health_check_app = Router::new().route("/health_check", get(|| async { "ok" }));

    match hostname.as_str() {
        "health.check" => health_check_app.oneshot(request),
        _ => primary_app.oneshot(request),
    }
    .await
}

async fn handle_error(_err: std::io::Error) -> impl IntoResponse {
    (StatusCode::INTERNAL_SERVER_ERROR, "Something went wrong...")
}
