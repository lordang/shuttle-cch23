use axum::{extract::Multipart, routing::post, Router};
use reqwest::StatusCode;
use tower_http::services::ServeDir;

pub fn routes() -> Router {
    Router::new()
        .route("/red_pixels", post(task2))
        .nest_service("/assets", ServeDir::new("assets"))
}

pub async fn task2(mut multipart: Multipart) -> Result<String, StatusCode> {
    let mut counts = 0;
    while let Some(field) = multipart
        .next_field()
        .await
        .map_err(|_| StatusCode::BAD_REQUEST)?
    {
        let data = field.bytes().await.map_err(|_| StatusCode::BAD_REQUEST)?;
        let image = image::load_from_memory(&data).map_err(|_| StatusCode::BAD_REQUEST)?;
        counts += image
            .to_rgb32f()
            .pixels()
            .filter(|pixel| pixel[0] > pixel[1] + pixel[2])
            .count();
    }
    Ok(counts.to_string())
}
