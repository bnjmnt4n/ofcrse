use std::collections::HashMap;
use std::io::BufReader;
use std::net::SocketAddr;
use std::time::Duration;

mod error;

use crate::error::HttpError;
use axum::{
    body::{Body, BoxBody},
    extract::{FromRef, Host, Path, State},
    http::{header, Request, StatusCode},
    response::Response,
    routing::{any, get, get_service},
    Router,
};
use color_eyre::Report;
use hyper::{client::HttpConnector, HeaderMap, Uri};
use hyper_tls::HttpsConnector;
use tower::ServiceExt;
use tower_http::{
    services::{ServeDir, ServeFile},
    trace::TraceLayer,
};
use tracing::{info, Span};
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};

const DEFAULT_PORT: i32 = 3000;
const DEFAULT_SITE_URL: &str = "http://localhost:3000";
const DEFAULT_SHORTLINKS_FILE: &str = "shortlinks.json";

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

    let shortlinks_file =
        std::env::var("SHORTLINKS_FILE").unwrap_or(DEFAULT_SHORTLINKS_FILE.to_string());
    let shortlinks = read_shortlinks_from_file(shortlinks_file.clone()).unwrap_or_else(|_| {
        info!("Could not open shortlinks file {}", shortlinks_file);
        HashMap::new()
    });

    let app_state = AppState {
        site_url,
        goatcounter_url,
        goatcounter_host,
        shortlinks,
    };

    let tracing_layer = TraceLayer::new_for_http()
        .make_span_with(|request: &Request<Body>| {
            let headers = request.headers();
            let host = get_header(headers, "host");
            let user_agent = get_header(headers, "user-agent");
            let client_ip = get_header(headers, "fly-client-ip");
            let referer = get_header(headers, "referer");

            tracing::info_span!(
                "request",
                method = %request.method(),
                uri = %request.uri(),
                version = ?request.version(),
                ?host,
                ?client_ip,
                ?user_agent,
                ?referer,
            )
        })
        .on_request(|_: &Request<Body>, _: &Span| {
            tracing::info!("started processing request");
        })
        .on_response(
            |response: &Response<BoxBody>, latency: Duration, _span: &Span| {
                tracing::info!(
                    latency = format_args!("{} ms", latency.as_millis()),
                    status = %response.status(),
                    response_length = ?get_header(response.headers(), "content-length"),
                    "finished processing request"
                )
            },
        );

    let app = Router::new();
    // Required for both the standalone `/` path and the wildcard path.
    let app = initialize_app(app, app_state.clone(), "/");
    let app = initialize_app(app, app_state, "/*path");
    let app = app
        .fallback(|| async { Err::<(), HttpError>(HttpError::NotFound) })
        .layer(tracing_layer);

    axum::Server::bind(&address)
        .serve(app.into_make_service())
        .await?;

    Ok(())
}

fn get_header<'a>(headers: &'a HeaderMap, header_name: &'static str) -> Option<&'a str> {
    headers
        .get(header_name)
        .and_then(|header_value| header_value.to_str().map_or(None, Some))
}

type Client = hyper::client::Client<HttpsConnector<HttpConnector>, Body>;

fn initialize_app(app: Router, app_state: AppState, path: &str) -> Router {
    let https = HttpsConnector::new();
    let client = hyper::Client::builder().build(https);

    let primary_app = primary_router().with_state(PrimaryAppState {
        client,
        goatcounter: (app_state.goatcounter_url, app_state.goatcounter_host),
    });

    let music_shortlink = app_state.shortlinks.get("music").cloned();
    let music_shortlink_app = music_shortlink_router(music_shortlink).with_state(());

    let shortlink_app =
        shortlinks_router(app_state.site_url.clone(), app_state.shortlinks).with_state(());

    let redirect_to_primary_site =
        redirect_to_primary_site_router(app_state.site_url).with_state(());

    let health_check_app = health_check_router().with_state(());

    app.route(
        path,
        any(|Host(hostname): Host, request: Request<Body>| async move {
            let router = match hostname.as_str() {
                "health.check" => health_check_app,
                "l.ofcr.se" => shortlink_app,
                "music.ofcr.se" => music_shortlink_app,
                "ofcrse.fly.dev" => redirect_to_primary_site,
                _ => primary_app,
            };
            router.oneshot(request).await
        }),
    )
}

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

// Music shortlink.
fn music_shortlink_router(music_shortlink: Option<String>) -> Router {
    Router::new().route(
        "/",
        get(|| async move {
            music_shortlink.map_or(Err(HttpError::NotFound), |music_shortlink| {
                Ok((StatusCode::FOUND, [(header::LOCATION, music_shortlink)]))
            })
        }),
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

// Redirect back to primary app.
fn redirect_to_primary_site_router(site_url: String) -> Router {
    Router::new().fallback(|req: Request<Body>| async move {
        let path = req
            .uri()
            .path_and_query()
            .map(|v| v.as_str())
            .unwrap_or_else(|| req.uri().path());

        let uri = format!("{}{}", site_url, path);

        (StatusCode::MOVED_PERMANENTLY, [(header::LOCATION, uri)])
    })
}

// Health check app for fly.io.
fn health_check_router() -> Router {
    Router::new().route("/healthz", get(|| async { "ok" }))
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
