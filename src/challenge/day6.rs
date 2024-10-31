use axum::{routing::post, Json, Router};
use serde_json::{json, Value};

pub fn routes() -> Router {
    Router::new().route("/", post(task))
}

pub async fn task(s: String) -> Json<Value> {
    let mut str = &s[..];
    let mut elf = 0;
    let mut with_elf = 0;
    let mut without_elf = 0;
    while let Some(idx) = str.find("elf") {
        elf += 1;
        str = &str[idx + 1..];
    }
    str = &s[..];
    while let Some(idx) = str.find("elf on a shelf") {
        with_elf += 1;
        str = &str[idx + 1..];
    }
    str = &s[..];
    while let Some(idx) = str.find("shelf") {
        without_elf += 1;
        str = &str[idx + 1..];
    }
    without_elf -= with_elf;

    Json(json!({"elf": elf, "elf on a shelf": with_elf, "shelf with no elf on it": without_elf}))
}
