use axum::{http::StatusCode, routing::get, Router};

pub fn routes() -> Router {
    Router::new().route("/error", get(task2))
}

pub async fn task1() -> &'static str {
    ""
}

pub async fn task2() -> StatusCode {
    StatusCode::INTERNAL_SERVER_ERROR
}
