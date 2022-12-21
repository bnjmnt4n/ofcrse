use std::{
    collections::HashMap,
    io::BufReader,
    net::{IpAddr, Ipv4Addr, SocketAddr},
    sync::Arc,
};

use axum::{
    body::Body,
    extract::{FromRef, Host, Path, State},
    http::{header, Request, StatusCode},
    response::{IntoResponse, Redirect, Response},
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
const DEFAULT_SHORTLINKS_DB: &str = "shortlinks.json";

#[derive(Clone, FromRef)]
struct AppState {
    site_url: String,
    shortlinks: Arc<HashMap<String, String>>,
}

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

    let site_url = std::env::var("SITE_URL").unwrap_or(DEFAULT_SITE_URL.to_string());
    let shortlinks_database =
        std::env::var("SHORTLINKS_DB").unwrap_or(DEFAULT_SHORTLINKS_DB.to_string());
    let shortlinks = Arc::new(read_shortlinks_from_file(shortlinks_database).unwrap());
    let app_state = AppState {
        site_url,
        shortlinks,
    };

    let app = Router::new()
        .route("/", any(handler))
        .route("/*path", any(handler))
        .with_state(app_state);

    axum::Server::bind(&address)
        .serve(app.layer(TraceLayer::new_for_http()).into_make_service())
        .await
        .unwrap();
}

async fn handler(
    State(app_state): State<AppState>,
    Host(hostname): Host,
    request: Request<Body>,
) -> impl IntoResponse {
    // Primary static file server.
    let file_server = ServeDir::new("dist").not_found_service(ServeFile::new("dist/404.html"));
    let primary_app =
        Router::new().fallback_service(get_service(file_server).handle_error(handle_error));

    // Music shortlink.
    let music_shortlink = app_state.shortlinks.get("music").unwrap().clone();
    let music_shortlink_app = Router::new().route(
        "/",
        get(|| async move { (StatusCode::FOUND, [(header::LOCATION, music_shortlink)]) }),
    );

    // All shortlinks.
    let site_url = app_state.site_url.clone();
    let shortlink_app = Router::new()
        .route("/", get(|| async move { Redirect::temporary(&site_url) }))
        .route("/*path", get(handle_shortlink))
        .with_state(app_state.shortlinks);

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

async fn handle_shortlink(
    State(shortlinks): State<Arc<HashMap<String, String>>>,
    Path((_, shortlink)): Path<(String, String)>,
) -> Response {
    if let Some(url) = shortlinks.get(&shortlink) {
        (StatusCode::FOUND, [(header::LOCATION, url)]).into_response()
    } else {
        StatusCode::NOT_FOUND.into_response()
    }
}

async fn handle_error(_err: std::io::Error) -> impl IntoResponse {
    (StatusCode::INTERNAL_SERVER_ERROR, "Something went wrong...")
}

fn read_shortlinks_from_file<P: AsRef<std::path::Path>>(
    path: P,
) -> Result<HashMap<String, String>, Box<dyn std::error::Error>> {
    let file = std::fs::File::open(path)?;
    let reader = BufReader::new(file);

    let shortlinks: HashMap<String, String> = serde_json::from_reader(reader)?;
    Ok(shortlinks)
}
