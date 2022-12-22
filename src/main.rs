use std::{collections::HashMap, io::BufReader, net::SocketAddr};

use axum::{
    body::Body,
    extract::{Host, Path},
    http::{header, Request, StatusCode},
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
const DEFAULT_SHORTLINKS_DB: &str = "shortlinks.json";

#[derive(Clone)]
struct AppState {
    site_url: String,
    shortlinks: HashMap<String, String>,
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
    let address = SocketAddr::from(([0, 0, 0, 0], port));

    let site_url = std::env::var("SITE_URL").unwrap_or(DEFAULT_SITE_URL.to_string());
    let shortlinks_database =
        std::env::var("SHORTLINKS_DB").unwrap_or(DEFAULT_SHORTLINKS_DB.to_string());
    let shortlinks = read_shortlinks_from_file(shortlinks_database).unwrap();
    let app_state = AppState {
        site_url,
        shortlinks,
    };

    let app = Router::new();
    // Required for both the standalone `/` path and the wildcard path.
    let app = initialize_app(app, app_state.clone(), "/");
    let app = initialize_app(app, app_state, "/*path");
    let app = app.fallback_service(
        // TODO: serve with 404 status code.
        get_service(ServeFile::new("dist/404.html")).handle_error(handle_error),
    );
    let app = app.layer(TraceLayer::new_for_http());

    axum::Server::bind(&address)
        .serve(app.into_make_service())
        .await
        .unwrap();
}

fn initialize_app(app: Router, app_state: AppState, path: &str) -> Router {
    let primary_app = primary_router();

    let music_shortlink = app_state.shortlinks.get("music").unwrap().clone();
    let music_shortlink_app = music_shortlink_router(music_shortlink);

    let shortlink_app = shortlinks_router(app_state.site_url, app_state.shortlinks);

    let health_check_app = health_check_router();

    app.route(
        path,
        any(|Host(hostname): Host, request: Request<Body>| async move {
            let router = match hostname.as_str() {
                "health.check" => health_check_app,
                "l.ofcr.se" => shortlink_app,
                "music.ofcr.se" => music_shortlink_app,
                _ => primary_app,
            };
            router.oneshot(request).await
        }),
    )
}

// Primary static file server.
fn primary_router() -> Router {
    // Primary static file server.
    let file_server = ServeDir::new("dist").not_found_service(ServeFile::new("dist/404.html"));
    Router::new().fallback_service(get_service(file_server).handle_error(handle_error))
}

// Music shortlink.
fn music_shortlink_router(music_shortlink: String) -> Router {
    Router::new().route(
        "/",
        get(|| async move { (StatusCode::FOUND, [(header::LOCATION, music_shortlink)]) }),
    )
}

// All shortlinks.
fn shortlinks_router(site_url: String, shortlinks: HashMap<String, String>) -> Router {
    Router::new()
        .route("/", get(|| async move { Redirect::temporary(&site_url) }))
        .route(
            "/*path",
            get(|Path((_, shortlink)): Path<(String, String)>| async move {
                if let Some(url) = shortlinks.get(&shortlink) {
                    (StatusCode::FOUND, [(header::LOCATION, url)]).into_response()
                } else {
                    StatusCode::NOT_FOUND.into_response()
                }
            }),
        )
}

// Health check app for fly.io.
fn health_check_router() -> Router {
    Router::new().route("/health_check", get(|| async { "ok" }))
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
