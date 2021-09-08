use crate::DwdError;
use lazy_static::lazy_static;
use regex::Regex;
use serde::Serialize;
use std::str::FromStr;

lazy_static! {
    static ref UNDEF_REGEX: Regex = Regex::new("^-*$").unwrap();
}

#[derive(Serialize)]
pub struct MosmixStation {
    clu: i32,
    cof_x: Option<u32>,
    id: String,
    icao: Option<String>,
    name: String,
    latitude: f32,
    longitude: f32,
    elevation: i32,
    hmod_h: Option<i32>,
    station_type: String,
}

pub async fn get_mosmix_stations() -> Result<Vec<MosmixStation>, DwdError> {
    let res = reqwest::get("https://www.dwd.de/DE/leistungen/met_verfahren_mosmix/mosmix_stationskatalog.cfg?view=nasPublication")
        .await.map_err(|_| DwdError::NotFound)?
        .error_for_status().map_err(|_| DwdError::NotFound)?
        .text()
        .await.map_err(|_| DwdError::NotFound)?;

    Ok(parse_mosmix_cfg(res))
}

pub fn parse_mosmix_cfg(data: String) -> Vec<MosmixStation> {
    data.split("\n\n")
        .map(|table| {
            let mut iter = table.trim().split("\n").skip(2);
            // get the line with "=== --- === ..." and measure the characters/width
            let attr_width: Vec<usize> = iter
                .next()
                .unwrap_or("")
                .split_whitespace()
                .map(|a| a.len())
                .collect();
            if attr_width.len() != 10 {
                return None;
            }
            Some(
                iter.map(move |row| {
                    let mut values = Vec::with_capacity(10);
                    let mut char_pos = 0;
                    for i in 0..10 {
                        let width = attr_width[i];
                        if char_pos + width > row.len() {
                            return None;
                        }
                        let text = &row[char_pos..(char_pos + width)];
                        values.push(if UNDEF_REGEX.is_match(text) {
                            None
                        } else {
                            Some(text.trim())
                        });
                        char_pos += width + 1;
                    }
                    Some(MosmixStation {
                        clu: i32::from_str(values[0].unwrap_or("0")).unwrap_or(0),
                        cof_x: values[1].map(|v| u32::from_str(v).ok()).unwrap_or(None),
                        id: values[2].unwrap_or("").to_owned(),
                        icao: values[3].map(|v| v.to_owned()),
                        name: values[4].unwrap_or("").to_owned(),
                        latitude: f32::from_str(values[5].unwrap_or("0")).unwrap_or(0f32),
                        longitude: f32::from_str(values[6].unwrap_or("0")).unwrap_or(0f32),
                        elevation: i32::from_str(values[7].unwrap_or("0")).unwrap_or(0),
                        hmod_h: values[8].map(|v| i32::from_str(v).ok()).unwrap_or(None),
                        station_type: values[9].unwrap_or("").to_owned(),
                    })
                })
                .filter_map(|o| o),
            )
        })
        .filter_map(|o| o)
        .flatten()
        .collect()
}
