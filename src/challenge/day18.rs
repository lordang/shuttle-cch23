use axum::{
    extract::{Path, State},
    routing::{get, post},
    Json, Router,
};
use reqwest::StatusCode;
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};
use sqlx::FromRow;

use crate::{
    db::{reset_orders, reset_regions},
    AppState,
};

use super::day13::insert_orders;

#[derive(Serialize, Deserialize, FromRow)]
pub struct Region {
    pub id: i32,
    pub name: String,
}

#[derive(Serialize)]
pub struct TopN {
    region: String,
    top_gifts: Vec<String>,
}

pub fn routes(state: AppState) -> Router {
    Router::new()
        .route("/reset", post(reset))
        .route("/orders", post(insert_orders))
        .route("/regions", post(insert_regions))
        .route("/regions/total", get(total_regions))
        .route("/regions/top_list/:number", get(topn_per_region))
        .with_state(state)
}

pub async fn reset(State(state): State<AppState>) -> Result<(), StatusCode> {
    reset_orders(&state.pool)
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    reset_regions(&state.pool)
        .await
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    Ok(())
}

pub async fn insert_regions(State(state): State<AppState>, Json(regions): Json<Vec<Region>>) {
    for region in regions {
        sqlx::query!(
            "INSERT INTO regions (id, name) VALUES ($1, $2) RETURNING id",
            region.id,
            region.name,
        )
        .fetch_one(&state.pool)
        .await
        .unwrap();
    }
}

pub async fn total_regions(State(state): State<AppState>) -> Json<Value> {
    let result = sqlx::query!(
        r#"
        SELECT SUM(orders.quantity) as total, regions.name 
        FROM orders JOIN regions 
        ON orders.region_id = regions.id 
        GROUP BY regions.id 
        ORDER BY regions.name
        "#
    )
    .fetch_all(&state.pool)
    .await
    .unwrap();
    let result: Vec<Value> = result
        .iter()
        .map(|x| json!({ "region": x.name, "total": x.total.unwrap_or(0) }))
        .collect();
    Json(json!(result))
}

pub async fn topn_per_region(
    State(state): State<AppState>,
    Path(number): Path<i64>,
) -> Json<Vec<TopN>> {
    let regions = sqlx::query!(
        r#"
        SELECT id, name
        FROM regions
        ORDER BY name
        "#
    )
    .fetch_all(&state.pool)
    .await
    .unwrap();

    let mut result: Vec<TopN> = Vec::new();
    for region in regions {
        let top_gifts = sqlx::query!(
            r#"
        SELECT SUM(orders.quantity) as total, regions.name, orders.gift_name
        FROM orders JOIN regions 
        ON orders.region_id = regions.id 
        WHERE regions.id = $1
        GROUP BY regions.id, orders.gift_name
        ORDER BY total DESC, orders.gift_name
        limit $2
        "#,
            region.id,
            number
        )
        .fetch_all(&state.pool)
        .await
        .unwrap();

        let top_gifts: Vec<String> = top_gifts
            .into_iter()
            .map(|x| x.gift_name.unwrap())
            .collect();

        result.push(TopN {
            region: region.name.unwrap(),
            top_gifts,
        });
    }
    Json(result)
}
