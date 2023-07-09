use crate::DwdError;
use chrono::{prelude::DateTime, NaiveDateTime, Utc};
use lazy_static::lazy_static;
use regex::Regex;
use serde::Serialize;
use serde_json::Value;
use std::{collections::HashMap, io::Cursor, str::FromStr};

lazy_static! {
    static ref UNDEF_REGEX: Regex = Regex::new("^-*$").unwrap();
}

#[derive(Serialize)]
pub struct WeatherReport {
    units: HashMap<String, String>,
    data: Vec<HashMap<String, Value>>,
}

pub async fn get_weather_report(station: String) -> Result<WeatherReport, DwdError> {
    let station = if station.len() < 5 {
        format!("{}{}", station, "_".repeat(5 - station.len()))
    } else {
        station
    };
    let url = format!(
        "https://opendata.dwd.de/weather/weather_reports/poi/{}-BEOB.csv",
        station
    );
    let res = reqwest::get(&url)
        .await
        .map_err(|_| DwdError::NoReport)?
        .error_for_status()
        .map_err(|_| DwdError::NoReport)?
        .bytes()
        .await
        .map_err(|_| DwdError::NoReport)?;
    let reader = Cursor::new(res);

    parse_weather_report(reader)
}

pub fn parse_weather_report<R: std::io::Read>(report: R) -> Result<WeatherReport, DwdError> {
    let mut reader = csv::ReaderBuilder::new();
    let reader = reader.has_headers(false).delimiter(b';');
    let mut csv_file = reader.from_reader(report);
    let mut iter = csv_file.records();

    let properties = iter
        .next()
        .ok_or(DwdError::NoHeaderRow)?
        .map_err(|_| DwdError::BadCsvLine)?;
    let properties: Vec<&str> = properties.iter().skip(2).collect();

    let param_length = properties.len();

    let units = {
        let mut unit_map = HashMap::<String, String>::with_capacity(param_length);
        for (unit_idx, unit) in iter
            .next()
            .ok_or(DwdError::NoUnitRow)?
            .map_err(|_| DwdError::BadCsvLine)?
            .iter()
            .skip(2)
            .enumerate()
        {
            unit_map.insert(properties[unit_idx].to_owned(), unit.to_owned());
        }
        unit_map
    };

    // skip german comments?!
    iter.next();

    if units.len() != properties.len() {
        return Err(DwdError::UnitMismatch);
    }

    let data: Vec<HashMap<String, Value>> = iter
        .filter_map(|record| {
            if record.is_err() {
                return None;
            }
            let record = record.unwrap();
            let mut record = record.iter();
            let date = record.next()?;
            let time = record.next()?;

            let timestamp = DateTime::<Utc>::from_utc(
                NaiveDateTime::parse_from_str(&format!("{} {}", date, time), "%d.%m.%y %H:%M")
                    .ok()?,
                Utc,
            )
            .timestamp_millis();

            let mut map = HashMap::<String, Value>::with_capacity(param_length + 1);
            map.insert("timestamp".to_owned(), Value::from(timestamp));
            for (idx, value) in record.enumerate() {
                let prop = properties[idx].to_owned();
                if !UNDEF_REGEX.is_match(value) {
                    map.insert(
                        prop,
                        if let Ok(value) = f32::from_str(&value.replace(',', ".")) {
                            Value::from(value)
                        } else {
                            Value::from(value)
                        },
                    );
                }
            }

            Some(map)
        })
        .collect();

    Ok(WeatherReport { units, data })
}
