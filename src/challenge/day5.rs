use axum::{
    extract::{Json, Query},
    routing::post,
    Router,
};
use serde::Deserialize;

#[derive(Deserialize)]
pub struct Pagination {
    #[serde(default = "default_offset")]
    offset: u32,
    limit: Option<u32>,
    split: Option<u32>,
}

pub fn routes() -> Router {
    Router::new().route("/", post(task))
}

fn default_offset() -> u32 {
    0
}

pub async fn task(
    Query(pagination): Query<Pagination>,
    Json(contents): Json<Vec<String>>,
) -> String {
    let start = pagination.offset as usize;
    let mut end = contents.len();

    if let Some(limit) = pagination.limit {
        end = end.min(start + limit as usize);
    }

    let result: String;
    if let Some(split) = pagination.split {
        result = format!(
            "{:?}",
            contents[start..end]
                .chunks(split as usize)
                .map(|s| s.into())
                .collect::<Vec<Vec<String>>>()
        );
    } else {
        result = format!("{:?}", contents[start..end].to_vec());
    }
    result
}
