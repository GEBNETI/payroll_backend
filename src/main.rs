use std::{env, net::SocketAddr, str::FromStr};

use tracing::{error, info};
use tracing_subscriber::{EnvFilter, filter::Directive};

#[tokio::main]
async fn main() {
    let base_filter = EnvFilter::try_from_default_env().unwrap_or_else(|_| EnvFilter::new("info"));
    let filter = base_filter.add_directive(Directive::from_str("tower_http=info").unwrap());

    tracing_subscriber::fmt().with_env_filter(filter).init();

    let port = env::var("PORT")
        .ok()
        .and_then(|value| value.parse().ok())
        .unwrap_or(3000);
    let addr = SocketAddr::from(([0, 0, 0, 0], port));

    let listener = tokio::net::TcpListener::bind(addr)
        .await
        .expect("failed to bind address");

    let listen_addr = listener.local_addr().expect("addr");
    info!("Listening on http://{listen_addr}");

    if let Err(err) = nomina::server::run(listener).await {
        error!("server error: {err}");
    }
}
