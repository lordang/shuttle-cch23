use axum::{
    extract::{rejection::JsonRejection, FromRequest},
    http::StatusCode,
    response::{IntoResponse, Response},
    routing::post,
    Json, Router,
};
use emojis;
use itertools::izip;
use regex::Regex;
use serde::Serialize;
use serde_json::{json, Value};
use sha2::{Digest, Sha256};

// https://github.com/tokio-rs/axum/blob/main/examples/error-handling/src/main.rs

#[derive(FromRequest)]
#[from_request(via(axum::Json), rejection(AppError))]
pub struct AppJson<T>(T);

impl<T> IntoResponse for AppJson<T>
where
    Json<T>: IntoResponse,
{
    fn into_response(self) -> Response {
        Json(self.0).into_response()
    }
}

#[derive(thiserror::Error, Debug)]
pub enum AppError {
    #[error("Json error: {0}")]
    JsonRejection(#[from] JsonRejection),
    #[error("8 chars")]
    TooShort,
    #[error("more types of chars")]
    NotEnoughCharTypes,
    #[error("55555")]
    NotEnoughDigits,
    #[error("math is hard")]
    NotAddsUp2023,
    #[error("not joyful enough")]
    NoJoyOrder,
    #[error("illegal: no sandwich")]
    NotRepeatBetween,
    #[error("outranged")]
    NoUnicodeInRange,
    #[error("😳")]
    NoEmoji,
    #[error("not a coffee brewer")]
    IllegalHashEnd,
}

impl IntoResponse for AppError {
    fn into_response(self) -> Response {
        let (status, reason) = match self {
            AppError::JsonRejection(rejection) => (rejection.status(), rejection.body_text()),
            AppError::TooShort => (StatusCode::BAD_REQUEST, self.to_string()),
            AppError::NotEnoughCharTypes => (StatusCode::BAD_REQUEST, self.to_string()),
            AppError::NotEnoughDigits => (StatusCode::BAD_REQUEST, self.to_string()),
            AppError::NotAddsUp2023 => (StatusCode::BAD_REQUEST, self.to_string()),
            AppError::NoJoyOrder => (StatusCode::NOT_ACCEPTABLE, self.to_string()),
            AppError::NotRepeatBetween => {
                (StatusCode::UNAVAILABLE_FOR_LEGAL_REASONS, self.to_string())
            }
            AppError::NoUnicodeInRange => (StatusCode::RANGE_NOT_SATISFIABLE, self.to_string()),
            AppError::NoEmoji => (StatusCode::UPGRADE_REQUIRED, self.to_string()),
            AppError::IllegalHashEnd => (StatusCode::IM_A_TEAPOT, self.to_string()),
        };
        (
            status,
            AppJson(AppResponse {
                result: "naughty".to_string(),
                reason,
            }),
        )
            .into_response()
    }
}

#[derive(Serialize, Debug, Clone)]
pub struct AppResponse {
    result: String,
    reason: String,
}

pub fn routes() -> Router {
    Router::new()
        .route("/nice", post(task1))
        .route("/game", post(task2))
}

fn validate(s: &str) -> bool {
    let cond1 = s.chars().filter(|c| "aeiouy".contains(*c)).count() >= 3;
    let cond2 = s
        .chars()
        .zip(s.chars().skip(1))
        .any(|(a, b)| a == b && a.is_alphabetic());
    let cond3 = !s.contains("ab") && !s.contains("cd") && !s.contains("pq") && !s.contains("xy");

    cond1 && cond2 && cond3
}

fn validate_length(content: &str) -> Result<(), AppError> {
    if content.len() < 8 {
        return Err(AppError::TooShort);
    }
    Ok(())
}

fn validate_char_types(content: &str) -> Result<(), AppError> {
    if !Regex::new(r"[A-Z]+").unwrap().is_match(content)
        || !Regex::new(r"[a-z]+").unwrap().is_match(content)
        || !Regex::new(r"[0-9]+").unwrap().is_match(content)
    {
        return Err(AppError::NotEnoughCharTypes);
    }
    Ok(())
}

fn validate_digits(content: &str) -> Result<(), AppError> {
    if content.chars().filter(|c| c.is_numeric()).count() < 5 {
        return Err(AppError::NotEnoughDigits);
    }
    Ok(())
}

fn validate_integers(content: &str) -> Result<(), AppError> {
    let mut value = 0;
    for mat in Regex::new(r"\d+").unwrap().find_iter(content) {
        value += content[mat.start()..mat.end()].parse::<i32>().unwrap();
    }
    if value != 2023 {
        return Err(AppError::NotAddsUp2023);
    }
    Ok(())
}

fn validate_joy(content: &str) -> Result<(), AppError> {
    if let Some(idx) = content.find("y") {
        if content[idx + 1..].contains("o") || content[idx + 1..].contains("j") {
            return Err(AppError::NoJoyOrder);
        }
    } else {
        return Err(AppError::NoJoyOrder);
    }
    if let Some(idx) = content.find("o") {
        if content[idx + 1..].contains("j") {
            return Err(AppError::NoJoyOrder);
        }
    } else {
        return Err(AppError::NoJoyOrder);
    }
    Ok(())
}

fn validate_repeat(content: &str) -> Result<(), AppError> {
    if !izip!(
        content.chars(),
        content.chars().skip(1),
        content.chars().skip(2)
    )
    .any(|(a, b, c)| a.is_alphabetic() && b.is_alphabetic() && a == c && b != c)
    {
        return Err(AppError::NotRepeatBetween);
    }
    Ok(())
}

fn validate_unicode(content: &str) -> Result<(), AppError> {
    if content
        .chars()
        .all(|c| !('\u{2980}'..'\u{2BFF}').contains(&c))
    {
        return Err(AppError::NoUnicodeInRange);
    }
    Ok(())
}

fn validate_emoji(content: &str) -> Result<(), AppError> {
    if content
        .chars()
        .all(|c| emojis::get(c.to_string().as_str()).is_none())
    {
        return Err(AppError::NoEmoji);
    }
    Ok(())
}

fn validate_sha256_hash(content: &str) -> Result<(), AppError> {
    let hash = Sha256::digest(content);
    let hex_hash = base16ct::lower::encode_string(&hash);
    let end = hex_hash.chars().last();
    if end.is_none() || end.unwrap() != 'a' {
        return Err(AppError::IllegalHashEnd);
    }
    Ok(())
}

fn check_rules(content: &str) -> Result<(), AppError> {
    validate_length(content)?;
    validate_char_types(content)?;
    validate_digits(content)?;
    validate_integers(content)?;
    validate_joy(content)?;
    validate_repeat(content)?;
    validate_unicode(content)?;
    validate_emoji(content)?;
    validate_sha256_hash(content)?;

    Ok(())
}

pub async fn task1(Json(payload): Json<Value>) -> (StatusCode, Json<Value>) {
    if validate(payload["input"].as_str().unwrap_or("")) {
        (StatusCode::OK, Json(json!({"result": "nice"})))
    } else {
        (StatusCode::BAD_REQUEST, Json(json!({"result": "naughty"})))
    }
}

pub async fn task2(Json(payload): Json<Value>) -> Result<AppJson<AppResponse>, AppError> {
    check_rules(payload["input"].as_str().unwrap_or(""))?;
    Ok(AppJson(AppResponse {
        result: "nice".to_string(),
        reason: "that's a nice password".to_string(),
    }))
}
