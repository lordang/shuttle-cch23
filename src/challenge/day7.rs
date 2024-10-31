use axum::{
    http::{header, HeaderMap, StatusCode},
    routing::get,
    Json, Router,
};
use base64::{engine, Engine};
use serde::{Deserialize, Serialize};
use std::{collections::HashMap, str};

#[derive(Serialize, Deserialize, Debug)]
struct Ingredient {
    flour: u64,
    sugar: u64,
    butter: u64,
    #[serde(rename = "baking powder")]
    baking_powder: u64,
    #[serde(rename = "chocolate chips")]
    chocolate_chips: u64,
}

#[derive(Serialize, Deserialize, Debug)]
struct TotalIngredient {
    recipe: HashMap<String, u64>,
    pantry: HashMap<String, u64>,
}

impl TotalIngredient {
    pub fn bake(&self) -> BakedIngredient {
        let cookies = self
            .recipe
            .iter()
            .filter(|(_, value)| **value != 0)
            .filter(|(key, _)| self.pantry.contains_key(*key))
            .map(|(key, value)| self.pantry.get(key).unwrap() / *value)
            .min()
            .unwrap_or(0);

        let mut pantry = self.pantry.clone();
        for (key, value) in pantry.iter_mut() {
            if let Some(recipe_value) = self.recipe.get(key) {
                *value -= *recipe_value * cookies;
            }
        }

        BakedIngredient { cookies, pantry }
    }
}

#[derive(Serialize, Deserialize, Debug)]
pub struct BakedIngredient {
    cookies: u64,
    pantry: HashMap<String, u64>,
}

pub fn routes() -> Router {
    Router::new()
        .route("/decode", get(task1))
        .route("/bake", get(task2))
}

fn get_cookie(header: HeaderMap) -> Result<String, StatusCode> {
    Ok(header
        .get(header::COOKIE)
        .and_then(|header| header.to_str().ok())
        .ok_or(StatusCode::BAD_REQUEST)?
        .to_string())
}

fn parse(recipe: &str) -> Result<String, StatusCode> {
    match engine::general_purpose::STANDARD.decode(recipe) {
        Ok(decoded) => Ok(str::from_utf8(&decoded).unwrap().to_string()),
        _ => Err(StatusCode::BAD_REQUEST),
    }
}

fn get_recipe(cookie: &str) -> &str {
    let recipe: Vec<&str> = cookie.split("recipe=").collect();
    recipe[1]
}

pub async fn task1(header: HeaderMap) -> Result<String, StatusCode> {
    let cookie = get_cookie(header)?;
    let recipe = get_recipe(&cookie);
    parse(recipe)
}

pub async fn task2(header: HeaderMap) -> Result<Json<BakedIngredient>, StatusCode> {
    let cookie = get_cookie(header)?;
    let recipe = parse(get_recipe(&cookie))?;
    let input: TotalIngredient = serde_json::from_str(&recipe).or(Err(StatusCode::BAD_REQUEST))?;
    let output = input.bake();
    Ok(Json(output))
}
