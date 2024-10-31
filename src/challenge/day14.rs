use askama::Template;
use axum::{
    routing::{post, Router},
    Json,
};
use serde_json::Value;

#[derive(Template)]
#[template(path = "day14.html")]
struct ContentTemplate<'a> {
    content: &'a str,
}
pub fn routes() -> Router {
    Router::new()
        .route("/unsafe", post(task1))
        .route("/safe", post(task2))
}
pub async fn task1(Json(payload): Json<Value>) -> String {
    let content = payload["content"].as_str().unwrap();
    format!(
        r#"<html>
  <head>
    <title>CCH23 Day 14</title>
  </head>
  <body>
    {}
  </body>
</html>"#,
        content
    )
}

pub async fn task2(Json(payload): Json<Value>) -> String {
    let content = payload["content"].as_str().unwrap();
    ContentTemplate { content }.render().unwrap()
}
