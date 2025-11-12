use std::{env, net::SocketAddr};

#[tokio::main]
async fn main() {
    let port = env::var("PORT")
        .ok()
        .and_then(|value| value.parse().ok())
        .unwrap_or(3000);
    let addr = SocketAddr::from(([0, 0, 0, 0], port));

    let listener = tokio::net::TcpListener::bind(addr)
        .await
        .expect("failed to bind address");

    println!(
        "Listening on http://{}",
        listener.local_addr().expect("addr")
    );

    if let Err(err) = nomina::server::run(listener).await {
        eprintln!("server error: {err}");
    }
}
