use axum::{extract::Path, routing::get, Router};

pub fn routes() -> Router {
    Router::new().route("/*path", get(task))
}

pub async fn task(Path(path): Path<String>) -> String {
    let nums: Vec<&str> = path.split("/").collect();
    let mut result: i32 = 0;

    for num in nums {
        result ^= num.parse::<i32>().unwrap();
    }
    result.pow(3).to_string()
}
