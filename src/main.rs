use std::net::{IpAddr, Ipv4Addr, SocketAddr};

use axum::{routing::get, Router};

const DEFAULT_PORT: i32 = 3000;

#[tokio::main]
async fn main() {
    let port: u16 = std::env::var("PORT")
        .unwrap_or(DEFAULT_PORT.to_string())
        .parse()
        .unwrap();
    let address = SocketAddr::new(IpAddr::V4(Ipv4Addr::new(0, 0, 0, 0)), port);

    let app = Router::new().route("/", get(|| async { "Hello, World!" }));

    axum::Server::bind(&address)
        .serve(app.into_make_service())
        .await
        .unwrap();
}
