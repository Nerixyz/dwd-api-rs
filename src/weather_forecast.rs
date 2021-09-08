use crate::{kml::deserialize_to_forecast, DwdError};
use serde::Serialize;
use std::{collections::HashMap, io::Cursor};
use zip::ZipArchive;

#[derive(Serialize)]
pub struct Forecast {
    pub name: String,
    pub description: String,
    pub issuer: String,
    pub generating_process: String,
    pub issue_time: u64,
    pub reference_models: Vec<ForecastReferenceModel>,
    pub coordinates: String,
    pub data: HashMap<&'static str, Vec<serde_json::Value>>,
    pub n_data_points: usize,
}

#[derive(Serialize)]
pub struct ForecastReferenceModel {
    pub name: String,
    pub reference_time: u64,
}

pub async fn get_forecast(station: &str) -> Result<Forecast, DwdError> {
    let url = format!("https://opendata.dwd.de/weather/local_forecasts/mos/MOSMIX_L/single_stations/{station}/kml/MOSMIX_L_LATEST_{station}.kmz", station = station);
    let res = reqwest::get(&url)
        .await
        .map_err(|_| DwdError::NotFound)?
        .error_for_status()
        .map_err(|_| DwdError::NotFound)?
        .bytes()
        .await
        .map_err(|_| DwdError::NotFound)?;

    actix_web::rt::task::spawn_blocking(move || {
        // unfortunately, zip is blocking

        let reader = Cursor::new(res);
        let mut zip = ZipArchive::new(reader).map_err(|_| DwdError::InvalidFile)?;
        let file = zip.by_index(0).map_err(|_| DwdError::InvalidFile)?;

        deserialize_to_forecast(file).map_err(|_| DwdError::ReadKmlError)
    })
    .await
    .map_err(|_| DwdError::InvalidFile)?
}
