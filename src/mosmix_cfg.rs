use crate::DwdError;
use itertools::Itertools;
use lazy_static::lazy_static;
use regex::Regex;
use serde::Serialize;
use std::str::FromStr;

lazy_static! {
    static ref UNDEF_REGEX: Regex = Regex::new("^-*$").unwrap();
}

#[derive(Serialize)]
pub struct MosmixStation {
    id: String,
    icao: Option<String>,
    name: String,
    latitude: f32,
    longitude: f32,
    elevation: i32,
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
        .flat_map(|table| {
            table.trim().split('\n').skip(2).filter_map(|row| {
                row.split_ascii_whitespace().collect_tuple().map(
                    |(id, icao, name, lat, lon, elevation)| MosmixStation {
                        id: id.to_owned(),
                        icao: if UNDEF_REGEX.is_match(icao) {
                            None
                        } else {
                            Some(icao.to_owned())
                        },
                        name: name.to_owned(),
                        latitude: f32::from_str(lat).unwrap_or(0f32),
                        longitude: f32::from_str(lon).unwrap_or(0f32),
                        elevation: i32::from_str(elevation).unwrap_or(0),
                    },
                )
            })
        })
        .collect()
}
