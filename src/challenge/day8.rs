use axum::{extract::Path, http::StatusCode, routing::get, Router};
use reqwest;
use serde_json::{self, Value};

const GRAVIT_ACCEL: f64 = 9.825;
const HEIGHT: f64 = 10.0;

pub fn routes() -> Router {
    Router::new()
        .route("/weight/:pokedex_number", get(task1))
        .route("/drop/:pokedex_number", get(task2))
}

async fn get_weight(pokedex_number: u32) -> anyhow::Result<f64> {
    let output = reqwest::get(format!(
        "https://pokeapi.co/api/v2/pokemon/{}",
        pokedex_number
    ))
    .await?
    .text()
    .await?;

    let v: Value = serde_json::from_str(&output).unwrap();
    // hectograms to kilograms
    Ok(v["weight"].as_f64().unwrap() / 10.0)
}

pub async fn task1(Path(pokedex_number): Path<u32>) -> Result<String, StatusCode> {
    let weight = get_weight(pokedex_number)
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    Ok(weight.to_string())
}

pub async fn task2(Path(pokedex_number): Path<u32>) -> Result<String, StatusCode> {
    let weight = get_weight(pokedex_number)
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    // 2gh = v^2
    let velocity = f64::sqrt(2.0 * HEIGHT * GRAVIT_ACCEL);
    let momentum = weight * velocity;
    Ok(momentum.to_string())
}
