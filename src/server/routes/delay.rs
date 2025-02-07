use axum::{extract::Path, routing::get, Router};
use tokio::time::{sleep, Duration};

pub fn build() -> Router {
    Router::new()
        .route("/", get(delay))
        .route("/{seconds}", get(specified_delay))
}

pub async fn delay() {
    sleep(Duration::from_secs(1)).await;
}
pub async fn specified_delay(Path(seconds): Path<f64>) {
    sleep(Duration::from_secs_f64(seconds)).await;
}
