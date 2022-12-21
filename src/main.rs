use std::net::{IpAddr, Ipv4Addr, SocketAddr};

use axum::{
    body::Body,
    extract::Host,
    http::{Request, StatusCode},
    response::{IntoResponse, Redirect},
    routing::{any, get, get_service},
    Router,
};
use tower::util::ServiceExt;
use tower_http::{
    services::{ServeDir, ServeFile},
    trace::TraceLayer,
};
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};

const DEFAULT_PORT: i32 = 3000;
const DEFAULT_SITE_URL: &str = "http://localhost:3000/";

#[tokio::main]
async fn main() {
    tracing_subscriber::registry()
        .with(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| "ofcrse=info,tower_http=debug".into()),
        )
        .with(tracing_subscriber::fmt::layer())
        .init();

    let port: u16 = std::env::var("PORT")
        .unwrap_or(DEFAULT_PORT.to_string())
        .parse()
        .unwrap();
    let address = SocketAddr::new(IpAddr::V4(Ipv4Addr::new(0, 0, 0, 0)), port);

    let app = Router::new()
        .route("/", any(handler))
        .route("/*path", any(handler));

    axum::Server::bind(&address)
        .serve(app.layer(TraceLayer::new_for_http()).into_make_service())
        .await
        .unwrap();
}

async fn handler(Host(hostname): Host, request: Request<Body>) -> impl IntoResponse {
    let site_url = std::env::var("SITE_URL").unwrap_or(DEFAULT_SITE_URL.to_string());

    // Primary static file server.
    let file_server = ServeDir::new("dist").not_found_service(ServeFile::new("dist/404.html"));
    let primary_app =
        Router::new().fallback_service(get_service(file_server).handle_error(handle_error));

    // Shortlinks.
    // TODO: read from JSON/SQLite database.
    let shortlink_app = Router::new()
        .route("/", get(|| async move { Redirect::temporary(&site_url) }))
        .route(
            "/*path",
            get(|| async { Redirect::temporary("https://ofcr.se/") }),
        );
    let music_shortlink_app = Router::new()
        .route(
            "/",
            get(|| async { Redirect::temporary("https://open.spotify.com") }),
        )
        .route(
            "/*path",
            get(|| async { Redirect::temporary("https://open.spotify.com") }),
        );

    // Health check app for fly.io.
    let health_check_app = Router::new().route("/health_check", get(|| async { "ok" }));

    match hostname.as_str() {
        "health.check" => health_check_app.oneshot(request),
        "l.ofcr.se" => shortlink_app.oneshot(request),
        "music.ofcr.se" => music_shortlink_app.oneshot(request),
        _ => primary_app.oneshot(request),
    }
    .await
}

async fn handle_error(_err: std::io::Error) -> impl IntoResponse {
    (StatusCode::INTERNAL_SERVER_ERROR, "Something went wrong...")
}
