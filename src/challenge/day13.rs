use axum::{
    extract::State,
    http::StatusCode,
    routing::{get, post},
    Json, Router,
};
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};
use sqlx::FromRow;

use crate::{db::reset_orders, AppState};

#[derive(Serialize, Deserialize, FromRow)]
pub struct Order {
    pub id: i32,
    pub region_id: i32,
    pub gift_name: String,
    pub quantity: i32,
}

pub fn routes(state: AppState) -> Router {
    Router::new()
        .route("/sql", get(task1))
        .route("/reset", post(reset))
        .route("/orders", post(insert_orders))
        .route("/orders/total", get(total_orders))
        .route("/orders/popular", get(popular_orders))
        .with_state(state)
}

pub async fn task1(State(state): State<AppState>) -> Result<String, StatusCode> {
    let result: i32 = sqlx::query_scalar("SELECT 20231213")
        .fetch_one(&state.pool)
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    Ok(result.to_string())
}

pub async fn reset(State(state): State<AppState>) -> Result<(), StatusCode> {
    reset_orders(&state.pool)
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    Ok(())
}

pub async fn insert_orders(State(state): State<AppState>, Json(orders): Json<Vec<Order>>) {
    for order in orders {
        sqlx::query!(
            "INSERT INTO orders (id, region_id, gift_name, quantity) VALUES ($1, $2, $3, $4) RETURNING id",
            order.id,
            order.region_id,
            order.gift_name,
            order.quantity
        )
        .fetch_one(&state.pool)
        .await
        .unwrap();
    }
}

pub async fn total_orders(State(state): State<AppState>) -> Json<Value> {
    let result = sqlx::query!("SELECT quantity FROM orders")
        .fetch_all(&state.pool)
        .await
        .unwrap();
    let result = result.iter().map(|x| x.quantity.unwrap_or(0)).sum::<i32>();
    Json(json!({ "total": result }))
}

pub async fn popular_orders(State(state): State<AppState>) -> Json<Value> {
    match sqlx::query!("SELECT gift_name, SUM(quantity) as total FROM orders GROUP BY gift_name ORDER BY total DESC LIMIT 1")
        .fetch_optional(&state.pool)
        .await.unwrap() {
            Some(result) => Json(json!({ "popular": result.gift_name })),
            None => Json(json!({ "popular": null })),
        }
}
