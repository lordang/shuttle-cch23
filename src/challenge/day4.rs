use axum::{extract::Json, routing::post, Router};
use serde::{Deserialize, Serialize};

#[derive(Deserialize)]
pub struct Reindeer {
    #[allow(dead_code)]
    name: String,
    strength: i32,
}

#[derive(Deserialize)]
pub struct Reindeer2 {
    name: String,
    strength: i32,
    speed: f32,
    height: u32,
    antler_width: u32,
    snow_magic_power: u32,
    favorite_food: String,
    #[serde(alias = "cAnD13s_3ATeN-yesT3rdAy")]
    candies: u32,
}

#[derive(Serialize)]
pub struct Output {
    fastest: String,
    tallest: String,
    magician: String,
    consumer: String,
}

pub fn routes() -> Router {
    Router::new()
        .route("/strength", post(task1))
        .route("/contest", post(task2))
}

pub async fn task1(Json(reindeers): Json<Vec<Reindeer>>) -> String {
    let mut strengths = 0;
    for r in reindeers {
        strengths += r.strength;
    }
    strengths.to_string()
}

pub async fn task2(Json(reindeers): Json<Vec<Reindeer2>>) -> Json<Output> {
    let fastest = &reindeers
        .iter()
        .max_by(|x, y| x.speed.total_cmp(&y.speed))
        .unwrap();
    let tallest = &reindeers
        .iter()
        .max_by(|x, y| x.height.cmp(&y.height))
        .unwrap();
    let magician = &reindeers
        .iter()
        .max_by(|x, y| x.snow_magic_power.cmp(&y.snow_magic_power))
        .unwrap();
    let consumer = &reindeers
        .iter()
        .max_by(|x, y| x.candies.cmp(&y.candies))
        .unwrap();

    Json(Output {
        fastest: format!(
            "Speeding past the finish line with a strength of {} is {}",
            fastest.strength, fastest.name
        ),
        tallest: format!(
            "{} is standing tall with his {} cm wide antlers",
            tallest.name, tallest.antler_width
        ),
        magician: format!(
            "{} could blast you away with a snow magic power of {}",
            magician.name, magician.snow_magic_power
        ),
        consumer: format!(
            "{} ate lots of candies, but also some {}",
            consumer.name, consumer.favorite_food
        ),
    })
}
