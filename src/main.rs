use std::collections::HashMap;
use std::io::BufReader;
use std::net::SocketAddr;

mod error;

use crate::error::HttpError;
use axum::{
    body::Body,
    extract::{FromRef, Host, Path, State},
    http::uri::Scheme,
    http::{header, uri::Authority, Request, StatusCode},
    middleware::{self, Next},
    response::{IntoResponse, Response},
    routing::{any, get, get_service},
    Router,
};
use color_eyre::{eyre::Context, Report};
use hyper::{client::HttpConnector, Uri};
use tower::ServiceExt;
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
    goatcounter_url: String,
    goatcounter_host: String,
    shortlinks: HashMap<String, String>,
}

#[tokio::main]
async fn main() -> Result<(), color_eyre::Report> {
    color_eyre::install()?;
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

    let goatcounter_url = std::env::var("GOATCOUNTER_URL").unwrap_or(DEFAULT_SITE_URL.to_string());
    let goatcounter_host = Uri::try_from(&goatcounter_url)
        .unwrap()
        .host()
        .unwrap()
        .to_string();

    let shortlinks_database =
        std::env::var("SHORTLINKS_DB").unwrap_or(DEFAULT_SHORTLINKS_DB.to_string());
    let shortlinks = read_shortlinks_from_file(shortlinks_database)
        .wrap_err("Could not open shortlinks file")?;

    let app_state = AppState {
        site_url,
        goatcounter_url,
        goatcounter_host,
        shortlinks,
    };

    let app = Router::new();
    // Required for both the standalone `/` path and the wildcard path.
    let app = initialize_app(app, app_state.clone(), "/");
    let app = initialize_app(app, app_state, "/*path");
    // TODO: return 404 file.
    let app = app.fallback(|| async { Err::<(), HttpError>(HttpError::NotFound) });
    let app = app.layer(TraceLayer::new_for_http());

    axum::Server::bind(&address)
        .serve(app.into_make_service())
        .await?;

    Ok(())
}

fn initialize_app(app: Router, app_state: AppState, path: &str) -> Router {
    let primary_app = primary_router().with_state(PrimaryAppState {
        client: Client::new(),
        goatcounter: (app_state.goatcounter_url, app_state.goatcounter_host),
    });

    let music_shortlink = app_state.shortlinks.get("music").unwrap().clone();
    let music_shortlink_app = music_shortlink_router(music_shortlink).with_state(());

    let shortlink_app = shortlinks_router(app_state.site_url, app_state.shortlinks).with_state(());

    let health_check_app = health_check_router().with_state(());

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

type Client = hyper::client::Client<HttpConnector, Body>;

#[derive(Clone, FromRef)]
struct PrimaryAppState {
    client: Client,
    goatcounter: (String, String),
}

// Primary static file server.
fn primary_router() -> Router<PrimaryAppState> {
    let file_server = ServeDir::new("dist").not_found_service(ServeFile::new("dist/404.html"));
    Router::new()
        .route("/count", any(goatcounter_proxy))
        .route("/count/", any(goatcounter_proxy))
        .route("/count/*path", any(goatcounter_proxy))
        .fallback_service(get_service(file_server).handle_error(handle_error))
        .layer(middleware::from_fn(redirect_to_https))
}

#[axum::debug_handler(state = PrimaryAppState)]
async fn goatcounter_proxy(
    State(client): State<Client>,
    State((goatcounter_url, goatcounter_host)): State<(String, String)>,
    mut req: Request<Body>,
) -> Result<Response<Body>, HttpError> {
    let path = req
        .uri()
        .path_and_query()
        .map(|v| v.as_str())
        .unwrap_or_else(|| req.uri().path());
    let path = path.strip_prefix("/count").unwrap();

    let uri = format!("{}{}", goatcounter_url, path);
    *req.uri_mut() = Uri::try_from(uri)?;

    let headers_map = req.headers_mut();
    // Remove any unnecessary headers.
    headers_map.remove("fly-forwarded-port");
    headers_map.remove("fly-region");
    headers_map.remove("x-forwarded-proto");
    headers_map.remove("x-forwarded-port");
    headers_map.remove("x-forwarded-ssl");
    // `X-Forwarded-For` header will be forwarded.
    headers_map.insert("Host", goatcounter_host.parse().unwrap());
    headers_map
        .remove("fly-client-ip")
        .and_then(|real_ip| headers_map.insert("X-Real-IP", real_ip));

    Ok(client.request(req).await?)
}

// Redirect any non-HTTPS requests to HTTP.
async fn redirect_to_https<B>(
    Host(hostname): Host,
    req: Request<B>,
    next: Next<B>,
) -> Result<Response, HttpError> {
    let proto: &str = req
        .headers()
        .get("x-forwarded-proto")
        .map(|header| header.to_str().unwrap_or("https"))
        .unwrap_or("https");

    if proto == "http" {
        let mut parts = req.uri().clone().into_parts();
        parts.scheme = Some(Scheme::HTTPS);
        // Read from `Host` header since the incoming request's authority is empty.
        parts.authority = Some(Authority::try_from(&hostname[..])?);

        let uri: String = Uri::from_parts(parts)?.to_string();

        Ok((StatusCode::MOVED_PERMANENTLY, [(header::LOCATION, uri)]).into_response())
    } else {
        Ok(next.run(req).await)
    }
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
        .route(
            "/",
            get(|| async move { (StatusCode::FOUND, [(header::LOCATION, site_url)]) }),
        )
        .route(
            "/*path",
            get(|Path((_, shortlink)): Path<(String, String)>| async move {
                if let Some(url) = shortlinks.get(&shortlink) {
                    Ok((StatusCode::FOUND, [(header::LOCATION, url.clone())]))
                } else {
                    Err(HttpError::NotFound)
                }
            }),
        )
}

// Health check app for fly.io.
fn health_check_router() -> Router {
    Router::new().route("/health_check", get(|| async { "ok" }))
}

async fn handle_error(err: std::io::Error) -> HttpError {
    err.into()
}

fn read_shortlinks_from_file<P: AsRef<std::path::Path>>(
    path: P,
) -> Result<HashMap<String, String>, Report> {
    let file = std::fs::File::open(path)?;
    let reader = BufReader::new(file);

    let shortlinks: HashMap<String, String> = serde_json::from_reader(reader)?;
    Ok(shortlinks)
}
