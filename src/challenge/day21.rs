use axum::{extract::Path, http::StatusCode, routing::get, Router};

use s2::{cellid::CellID, latlng::LatLng};
use serde_json::Value;

pub fn routes() -> Router {
    Router::new()
        .route("/coords/:binary", get(convert_to_dms))
        .route("/country/:binary", get(convert_to_country))
}

fn get_cell_id(binary: String) -> CellID {
    CellID(
        u64::from_str_radix(&binary, 2)
            .map_err(|_| StatusCode::BAD_REQUEST)
            .unwrap(),
    )
}

fn get_degree_from_cell_id(cell_id: CellID) -> (f64, f64) {
    let coord = LatLng::from(cell_id);

    (coord.lat.deg(), coord.lng.deg())
}

// decimal degree to DMS
// https://en.wikipedia.org/wiki/Decimal_degrees#Example
fn degree_to_dms(degree: f64) -> (f64, f64, f64) {
    let d = degree.trunc();
    let m = ((degree - d) * 60.0).trunc();
    let s = (degree - d) * 3600.0 - (m * 60.0);

    (d, m, s)
}

pub async fn convert_to_dms(Path(binary): Path<String>) -> String {
    let (lat, lng) = get_degree_from_cell_id(get_cell_id(binary));

    let (lat_degree, lat_minute, lat_second) = degree_to_dms(lat);
    let (lng_degree, lng_minute, lng_second) = degree_to_dms(lng);

    let mut lat_dir = "N";
    let mut lng_dir = "E";
    if lat_degree < 0.0 {
        lat_dir = "S";
    }
    if lng_degree < 0.0 {
        lng_dir = "W";
    }

    format!(
        "{}°{}'{:.3}''{} {}°{}'{:.3}''{}",
        lat_degree.abs(),
        lat_minute.abs(),
        lat_second.abs(),
        lat_dir,
        lng_degree.abs(),
        lng_minute.abs(),
        lng_second.abs(),
        lng_dir,
    )
}

pub async fn convert_to_country(Path(binary): Path<String>) -> String {
    let (lat, lng) = get_degree_from_cell_id(get_cell_id(binary));

    let client = reqwest::Client::new();
    // overpass-api is_in query
    let query = format!(
        r#"[out:json]
[timeout:25];

is_in({}, {})->.a;
rel(pivot.a)[boundary=administrative][admin_level=2];

out tags;"#,
        lat, lng
    );
    // overpass api
    let res = client
        .post("https://overpass-api.de/api/interpreter")
        .body(query)
        .send()
        .await
        .unwrap()
        .text()
        .await
        .unwrap();
    let v: Value = serde_json::from_str(&res).unwrap();

    v["elements"][0]["tags"]["name:en"]
        .as_str()
        .unwrap()
        .to_string()
}
