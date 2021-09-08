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
        .map_err(|_| DwdError::NotFound)?
        .error_for_status()
        .map_err(|_| DwdError::NotFound)?
        .bytes()
        .await
        .map_err(|_| DwdError::NotFound)?;
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
        .ok_or(DwdError::InvalidFile)?
        .map_err(|_| DwdError::InvalidFile)?;
    let properties: Vec<&str> = properties.iter().skip(2).collect();

    let param_length = properties.len();

    let units = {
        let mut unit_map = HashMap::<String, String>::with_capacity(param_length);
        let mut unit_idx = 0;
        for unit in iter
            .next()
            .ok_or(DwdError::InvalidFile)?
            .map_err(|_| DwdError::InvalidFile)?
            .iter()
            .skip(2)
        {
            unit_map.insert(properties[unit_idx].clone().to_owned(), unit.to_owned());
            unit_idx += 1;
        }
        unit_map
    };

    // skip german comments?!
    iter.next();

    if units.len() != properties.len() {
        return Err(DwdError::InvalidFile);
    }

    let data: Vec<HashMap<String, Value>> = iter
        .map(|record| {
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
            let mut idx = 0;
            for value in record {
                let prop = properties[idx].clone().to_owned();
                if !UNDEF_REGEX.is_match(value) {
                    map.insert(
                        prop,
                        if let Ok(value) = f32::from_str(&value.replace(",", ".")) {
                            Value::from(value)
                        } else {
                            Value::from(value)
                        },
                    );
                }
                idx += 1;
            }

            Some(map)
        })
        .filter(|v| v.is_some())
        .map(|v| v.unwrap())
        .collect();

    Ok(WeatherReport { units, data })
}
