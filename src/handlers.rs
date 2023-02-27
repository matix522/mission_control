use axum::{routing::get, Router};
pub mod mission;
pub mod tag;

pub(crate) fn register() -> Router {
    Router::new()
        .merge(mission::register())
        .route("/", get(root))
}

// basic handler that responds with a static string
async fn root() -> &'static str {
    "Hello, World!"
}
