use axum::Router;
use std::io;
use tokio::net::TcpListener;

pub async fn run(listener: TcpListener) -> Result<(), io::Error> {
    let app = router();
    axum::serve(listener, app).await
}

pub fn router() -> Router {
    crate::routes::app_router()
}
