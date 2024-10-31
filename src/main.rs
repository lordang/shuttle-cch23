use axum::{routing::get, Router};
use challenge::{
    day1, day11, day12, day13, day14, day15, day18, day19, day20, day21, day22, day4, day5, day6,
    day7, day8, day_1,
};
use sqlx::PgPool;

mod challenge;
mod db;

#[derive(Clone)]
struct AppState {
    pool: PgPool,
}

impl AppState {
    fn new(pool: PgPool) -> Self {
        Self { pool }
    }
}

#[shuttle_runtime::main]
async fn main(#[shuttle_shared_db::Postgres] pool: PgPool) -> shuttle_axum::ShuttleAxum {
    let state = AppState::new(pool);

    let router = Router::new()
        .route("/", get(day_1::task1))
        .nest("/-1", day_1::routes())
        .nest("/1", day1::routes())
        .nest("/4", day4::routes())
        .nest("/5", day5::routes())
        .nest("/6", day6::routes())
        .nest("/7", day7::routes())
        .nest("/8", day8::routes())
        .nest("/11", day11::routes())
        .nest("/12", day12::routes())
        .nest("/13", day13::routes(state.clone()))
        .nest("/14", day14::routes())
        .nest("/15", day15::routes())
        .nest("/18", day18::routes(state.clone()))
        .nest("/19", day19::routes())
        .nest("/20", day20::routes())
        .nest("/21", day21::routes())
        .nest("/22", day22::routes());

    Ok(router.into())
}
