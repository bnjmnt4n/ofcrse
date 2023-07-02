use std::borrow::Cow;
use std::collections::HashMap;
use std::io::BufReader;
use std::net::SocketAddr;
use std::time::Duration;

mod error;

use crate::error::HttpError;
use axum::{
    body::{Body, BoxBody},
    extract::{ConnectInfo, FromRef, Host, Path, State},
    http::{header, HeaderValue, Request, StatusCode},
    middleware::{self, Next},
    response::{IntoResponse, Response},
    routing::{any, get, get_service},
    Router,
};
use color_eyre::Report;
use hyper::{client::HttpConnector, HeaderMap, Uri};
use hyper_rustls::HttpsConnector;
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
            let client_ip: Option<Cow<str>> = get_header(headers, "fly-client-ip")
                .map(Cow::Borrowed)
                .or_else(|| {
                    request
                        .extensions()
                        .get::<ConnectInfo<SocketAddr>>()
                        .map(|ConnectInfo(addr)| Cow::Owned(addr.to_string()))
                });
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
        .serve(app.into_make_service_with_connect_info::<SocketAddr>())
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
    let https_connector = hyper_rustls::HttpsConnectorBuilder::new()
        .with_webpki_roots()
        .https_or_http()
        .enable_http1()
        .build();
    let client = hyper::Client::builder().build(https_connector);

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
                "oftcour.se" | "www.oftcour.se" | "ofcrse.fly.dev" => redirect_to_primary_site,
                host => {
                    if host.ends_with(".ofcr.se") || host.ends_with(".oftcour.se") {
                        return Err(HttpError::NotFound);
                    } else {
                        primary_app
                    }
                }
            };
            Ok(router.oneshot(request).await)
        }),
    )
}

#[derive(Clone, FromRef)]
struct PrimaryAppState {
    client: Client,
    goatcounter: (String, String),
}

const CONTENT_TYPE_JS: &[u8] = b"application/javascript";
const CONTENT_TYPE_CSS: &[u8] = b"text/css";
const CONTENT_TYPE_WOFF2: &[u8] = b"font/woff2";

// Primary static file server.
fn primary_router() -> Router<PrimaryAppState> {
    let file_server = ServeDir::new("dist").not_found_service(ServeFile::new("dist/404.html"));

    async fn custom_middleware<B>(mut request: Request<B>, next: Next<B>) -> Response {
        // Remove trailing slashes.
        let path = request.uri().path();
        if path.ends_with("/") && path != "/" && !path.starts_with("/count") {
            let proto = get_header(request.headers(), "x-forwarded-proto").unwrap_or("http");
            let host = get_header(request.headers(), "host").unwrap_or("");
            let path = path.strip_suffix("/").unwrap();
            let query = request.uri().query();

            let uri = format!(
                "{}://{}{}{}{}",
                proto,
                host,
                path,
                query.map_or("", |_| "?"),
                query.unwrap_or("")
            );

            (StatusCode::TEMPORARY_REDIRECT, [(header::LOCATION, uri)]).into_response()
        } else {
            // Assume path is a directory if there is no `.` in the path.
            if !path.contains(".") {
                let query = request.uri().query();

                let uri = format!(
                    "{}/index.html{}{}",
                    path,
                    query.map_or("", |_| "?"),
                    query.unwrap_or("")
                );

                if let Ok(uri) = Uri::try_from(uri) {
                    *request.uri_mut() = uri
                }
            }

            let mut response = next.run(request).await;

            // Add `Cache-Control` rules for static assets (JS/CSS/WOFF2).
            // TODO: images?
            match response
                .headers()
                .get("content-type")
                .map(|header| header.as_bytes())
            {
                Some(CONTENT_TYPE_JS) | Some(CONTENT_TYPE_CSS) | Some(CONTENT_TYPE_WOFF2) => {
                    response
                        .headers_mut()
                        .insert("Cache-Control", "public, max-age=31536000".parse().unwrap());
                }
                _ => {}
            }

            response
        }
    }

    Router::new()
        .route("/count", any(goatcounter_proxy))
        .route("/count/", any(goatcounter_proxy))
        .route("/count/*path", any(goatcounter_proxy))
        .fallback_service(get_service(file_server))
        // TODO: Apply middleware only to file server.
        .layer(middleware::from_fn(custom_middleware))
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
        .or_else(|| {
            req.extensions()
                .get::<ConnectInfo<SocketAddr>>()
                .and_then(|ConnectInfo(addr)| {
                    HeaderValue::from_str(&addr.to_string()).map_or(None, Some)
                })
        })
        .and_then(|addr| req.headers_mut().insert("X-Real-IP", addr));

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

fn read_shortlinks_from_file<P: AsRef<std::path::Path>>(
    path: P,
) -> Result<HashMap<String, String>, Report> {
    let file = std::fs::File::open(path)?;
    let reader = BufReader::new(file);

    let shortlinks: HashMap<String, String> = serde_json::from_reader(reader)?;
    Ok(shortlinks)
}
