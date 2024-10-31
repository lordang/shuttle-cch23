use std::{collections::HashMap, sync::Arc, time::SystemTime};

use serde::Serialize;
use tokio::sync::Mutex;

use axum::{
    extract::{Path, State},
    http::StatusCode,
    routing::{get, post},
    Json, Router,
};
use chrono::prelude::*;
use ulid::Ulid;
use uuid::Uuid;

#[derive(Clone)]
pub struct SharedState {
    packets: Arc<Mutex<HashMap<String, SystemTime>>>,
}

impl SharedState {
    pub fn new() -> Self {
        Self {
            packets: Arc::new(Mutex::new(HashMap::new())),
        }
    }
}

#[derive(Serialize, Debug)]
pub struct Lsb {
    #[serde(rename(serialize = "christmas eve"))]
    eve: u8,
    weekday: u8,
    #[serde(rename(serialize = "in the future"))]
    future: u8,
    #[serde(rename(serialize = "LSB is 1"))]
    lsb: u8,
}

impl Lsb {
    pub fn new() -> Self {
        Self {
            eve: 0,
            weekday: 0,
            future: 0,
            lsb: 0,
        }
    }
}

pub fn routes() -> Router {
    let state = SharedState::new();
    Router::new()
        .route("/save/:packet_id", post(save))
        .route("/load/:packet_id", get(load))
        .route("/ulids", post(ulids_to_uuids))
        .route("/ulids/:weekday", post(task3))
        .with_state(state)
}

pub async fn load(
    Path(packet_id): Path<String>,
    State(state): State<SharedState>,
) -> Result<String, StatusCode> {
    let elapsed = state
        .packets
        .lock()
        .await
        .get(&packet_id)
        .map(|t| t.elapsed().unwrap().as_secs())
        .ok_or(StatusCode::NOT_FOUND)?;
    Ok(elapsed.to_string())
}

pub async fn save(
    Path(packet_id): Path<String>,
    State(state): State<SharedState>,
) -> Result<(), StatusCode> {
    state
        .packets
        .lock()
        .await
        .insert(packet_id, SystemTime::now());
    Ok(())
}

pub async fn ulids_to_uuids(
    Json(ulids): Json<Vec<String>>,
) -> Result<Json<Vec<String>>, StatusCode> {
    Ok(Json(
        ulids
            .iter()
            .map(|ulid| Uuid::from(Ulid::from_string(ulid.as_str()).unwrap()).to_string())
            .rev()
            .collect::<Vec<String>>(),
    ))
}

pub async fn task3(
    Path(weekday): Path<u8>,
    Json(ulids): Json<Vec<String>>,
) -> Result<Json<Lsb>, StatusCode> {
    let mut lsb = Lsb::new();

    let ulids: Vec<Ulid> = ulids
        .iter()
        .map(|ulid| Ulid::from_string(ulid).unwrap())
        .collect();
    for ulid in ulids {
        let dt = DateTime::from_timestamp(ulid.timestamp_ms() as i64 / 1000, 0).unwrap();
        if dt.month() == 12 && dt.day() == 24 {
            lsb.eve += 1;
        }
        if dt.weekday().num_days_from_monday() as u8 == weekday {
            lsb.weekday += 1;
        }
        if dt.timestamp()
            > SystemTime::now()
                .duration_since(SystemTime::UNIX_EPOCH)
                .unwrap()
                .as_secs() as i64
        {
            lsb.future += 1;
        }
        if ulid.to_bytes()[15] & 0b1 == 1 {
            lsb.lsb += 1;
        }
    }
    Ok(Json(lsb))
}
