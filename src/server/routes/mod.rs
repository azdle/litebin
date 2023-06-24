mod delay;

use axum::{routing::get, Router};
use maud::{html, Markup};

pub fn build() -> Router {
    let delay = delay::build();
    Router::new()
        .route("/", get(html_homepage))
        .route("/health", get(health_check))
        .nest("/delay", delay)
}

pub async fn html_homepage() -> Markup {
    html! {
        h1 { "Hello" }
    }
}

// just always returns a 200 OK for now, the server has no state, if it's up, it's working
pub async fn health_check() {}
