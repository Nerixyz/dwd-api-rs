use serde_xml_rs::from_reader;
use crate::weather_forecast::{Forecast, ForecastReferenceModel};
use chrono::DateTime;
use std::error::Error;
use std::collections::HashMap;
use std::str::FromStr;
use serde_json::Value;
use serde::Deserialize;
use lazy_static::lazy_static;
use maplit::hashmap;

#[derive(Deserialize, Debug)]
struct Kml {
    #[serde(rename = "Document")]
    document: KmlDocument
}

#[derive(Deserialize, Debug)]
struct KmlDocument {
    #[serde(rename = "ExtendedData")]
    extended_data: KmlExtDocument,
    #[serde(rename = "Placemark")]
    placemark: KmlPlacemark,
}

#[derive(Deserialize, Debug)]
struct KmlExtDocument {
    #[serde(rename="ProductDefinition")]
    product_definition: ProductDefinition,
}

#[derive(Deserialize, Debug)]
struct KmlPoint {
    #[serde(default)]
    coordinates: String,
}

#[derive(Deserialize, Debug)]
struct KmlPlacemark {
    name: String,
    description: String,
    #[serde(rename = "Point")]
    point: KmlPoint,
    #[serde(rename = "ExtendedData")]
    extended_data: KmlExtPlacemark,
}
#[derive(Deserialize, Debug)]
struct KmlExtPlacemark {
    #[serde(rename = "Forecast")]
    forecasts: Vec<DwdForecast>
}

#[derive(Deserialize, Debug)]
struct DwdForecast {
    #[serde(rename = "elementName")]
    element_name: String,
    #[serde(default)]
    value: String,
}

#[derive(Deserialize, Debug)]
struct DwdModel {
    name: String,
    #[serde(rename = "referenceTime")]
    reference_time: String
}

#[derive(Deserialize, Debug)]
struct ReferencedModel {
    #[serde(rename = "Model", default)]
    models: Vec<DwdModel>
}

#[derive(Deserialize, Debug)]
struct ForecastTimeSteps {
    #[serde(rename = "TimeStep", default)]
    time_steps: Vec<String>
}

#[derive(Deserialize, Debug)]
struct FormatCfg {
    #[serde(rename = "DefaultUndefSign", default)]
    default_undef_sign: String,
}

#[derive(Deserialize, Debug)]
struct ProductDefinition {
    #[serde(rename = "Issuer")]
    issuer: String,
    #[serde(rename = "ProductID")]
    product_id: String,
    #[serde(rename = "GeneratingProcess")]
    generating_process: String,
    #[serde(rename = "IssueTime")]
    issue_time: String,
    #[serde(rename = "ReferencedModel")]
    referenced_models: ReferencedModel,
    #[serde(rename = "ForecastTimeSteps")]
    forecast_time_steps: ForecastTimeSteps,
    #[serde(rename = "FormatCfg")]
    format_config: FormatCfg,
}

pub fn deserialize_to_forecast<R: std::io::Read>(reader: R) -> Result<Forecast, Box<dyn Error>> {
    let deserialized: Kml = from_reader(reader)?;
    let product_def = deserialized.document.extended_data.product_definition;

    let (data, n_data_points) =
        kml_to_forecast_data(
            &deserialized.document.placemark.extended_data.forecasts,
            &product_def.forecast_time_steps.time_steps
        )?;

    Ok(Forecast {
        issuer: product_def.issuer,
        name: deserialized.document.placemark.name,
        coordinates: deserialized.document.placemark.point.coordinates,
        description: deserialized.document.placemark.description,
        generating_process: product_def.generating_process,
        issue_time: DateTime::parse_from_rfc3339(&product_def.issue_time)?.timestamp_millis() as u64,
        reference_models: product_def.referenced_models.models.iter().map(|m| ForecastReferenceModel {
            name: m.name.clone(),
            reference_time: DateTime::parse_from_rfc3339(&m.reference_time.clone()).map(|d|d.timestamp_millis()).unwrap_or(0) as u64,
        }).collect(),
        data,
        n_data_points
    })
}

fn kml_to_forecast_data(forecasts: &[DwdForecast], time_steps: &[String]) -> Result<(HashMap<&'static str, Vec<Value>>, usize), Box<dyn Error>> {
    let mut json = HashMap::<&'static str, Vec<Value>>::new();
    let time_steps: Vec<Value> = time_steps.
        iter()
        .map(|s|
            Value::from(
                    DateTime::parse_from_rfc3339(s)
                        .map(|d| d.timestamp_millis())
                        .unwrap_or(0)))
        .collect();
    let n_time_steps = time_steps.len();
    json.insert("time_steps", time_steps);

    for forecast in forecasts {
        let values: Vec<Value> = forecast.value
            .split_whitespace()
            .map(|s| f64::from_str(s).map(Value::from).unwrap_or(Value::Null)).collect();
        if values.len() != n_time_steps {
            continue;
        }
        if let Some(key) = KML_ELEMENT_TO_JSON_KEY.get(&forecast.element_name) {
            json.insert(key, values);
        }
    }

    Ok((json, n_time_steps))
}

lazy_static! {
    static ref KML_ELEMENT_TO_JSON_KEY: HashMap<String, &'static str> = hashmap! {
        "TTT".to_owned() => "temp",
        "Td".to_owned() => "dew_point",
        "TX".to_owned() => "max_temp",
        "TN".to_owned() => "min_temp",
        "DD".to_owned() => "wind_direction",
        "FF".to_owned() => "wind_speed",
        "FX1".to_owned() => "max_wind_gust_1h",
        "FX3".to_owned() => "max_wind_gust_3h",
        "FXh".to_owned() => "max_wind_gust_12h",
        "RR1c".to_owned() => "precipitation_1h_significant_weather",
        "RR1".to_owned() => "precipitation_1h",
        "RR3c".to_owned() => "precipitation_3h_significant_weather",
        "RR3".to_owned() => "precipitation_3h",
        "RRS1c".to_owned() => "snow_rain_eq_1h",
        "RRS3c".to_owned() => "snow_rain_eq_3h",
        "ww".to_owned() => "significant_weather",
        "W1W2".to_owned() => "past_weather_6h",
        "N".to_owned() => "total_cloud_cover",
        "Neff".to_owned() => "effective_cloud_cover",
        "N05".to_owned() => "cloud_cover_500ft",
        "Nl".to_owned() => "low_cloud_cover",
        "Nm".to_owned() => "midlevel_cloud_cover",
        "Nh".to_owned() => "high_cloud_cover",
        "PPPP".to_owned() => "surface_pressure",
        "T5cm".to_owned() => "temp_5cm",
        "RadS3".to_owned() => "shortwave_radiation_3h",
        "Rad1h".to_owned() => "global_irradiance",
        "RadL3".to_owned() => "longwave_radiation_3h",
        "VV".to_owned() => "visibility",
        "SunD1".to_owned() => "sunshine_last_hour",
        "FXh25".to_owned() => "p_wind_gust_25kn_12h",
        "FXh40".to_owned() => "p_wind_gust_40kn_12h",
        "FXh55".to_owned() => "p_wind_gust_55kn_12h",
        "wwM".to_owned() => "p_fog_1h",
        "wwM6".to_owned() => "p_fog_6h",
        "wwMh".to_owned() => "p_fog_12h",
        "Rh00".to_owned() => "p_precipitation_0mm_12h",
        "R602".to_owned() => "p_precipitation_p2mm_6h",
        "Rh02".to_owned() => "p_precipitation_p2mm_12h",
        "Rd02".to_owned() => "p_precipitation_p2mm_24h",
        "Rh10".to_owned() => "p_precipitation_1mm_12h",
        "R650".to_owned() => "p_precipitation_5mm_6h",
        "Rh50".to_owned() => "p_precipitation_5mm_12h",
        "Rd50".to_owned() => "p_precipitation_5mm_24h",
        "TG".to_owned() => "min_temp_5cm_12h",
        "TM".to_owned() => "mean_temp_24h",
        "DRR1".to_owned() => "precipitation_duration_1h",
        "wwZ".to_owned() => "p_drizzle_1h",
        "wwD".to_owned() => "p_straitform_precipitation_1h",
        "wwC".to_owned() => "p_convective_precipitation_1h",
        "wwT".to_owned() => "p_thunderstorms_1h",
        "wwL".to_owned() => "p_liquid_precipitation_1h",
        "wwS".to_owned() => "p_solid_precipitation_1h",
        "wwF".to_owned() => "p_freezing_rain_1h",
        "wwP".to_owned() => "p_precipitation_1h",
        "VV10".to_owned() => "p_visibility_below_1km",
        "E_TTT".to_owned() => "e_temp",
        "E_FF".to_owned() => "e_wind_speed",
        "E_DD".to_owned() => "e_wind_direction",
        "E_Td".to_owned() => "e_dew_point",
        "RR6".to_owned() => "precipitation_6h",
        "RR6c".to_owned() => "precipitation_6h_significant_weather",
        "R600".to_owned() => "p_precipitation_0mm_6h",
        "R101".to_owned() => "p_precipitation_p1mm_1h",
        "R102".to_owned() => "p_precipitation_p2mm_1h",
        "R103".to_owned() => "p_precipitation_p3mm_1h",
        "R105".to_owned() => "p_precipitation_p5mm_1h",
        "R107".to_owned() => "p_precipitation_p7mm_1h",
        "R110".to_owned() => "p_precipitation_1mm_1h",
        "R120".to_owned() => "p_precipitation_2mm_1h",
        "SunD".to_owned() => "sunshine_duration_yesterday",
        "RSunD".to_owned() => "rel_sunshine_duration_24h",
        "PSd00".to_owned() => "p_rel_sunshine_duration_24h",
        "PSd30".to_owned() => "p_rel_sunshine_duration_30p_24h",
        "PSd60".to_owned() => "p_rel_sunshine_duration_60p_24h",
        "RRad1".to_owned() => "global_irradiance_1h",
        "PEvap".to_owned() => "potential_evapotranspiration_24h",
        "R130".to_owned() => "p_precipitation_3mm_1h",
        "R150".to_owned() => "p_precipitation_5mm_1h",
        "RR1o1".to_owned() => "p_precipitation_10mm_1h",
        "RR1w1".to_owned() => "p_precipitation_15mm_1h",
        "RR1u1".to_owned() => "p_precipitation_25mm_1h",
        "wwD6".to_owned() => "p_straightform_precipitation_6h",
        "wwC6".to_owned() => "p_convective_precipitation_6h",
        "wwT6".to_owned() => "p_thunderstorms_6h",
        "wwP6".to_owned() => "p_precipitation_6h",
        "wwL6".to_owned() => "p_liquid_precipitation_6h",
        "wwF6".to_owned() => "p_freezing_rain_6h",
        "wwS6".to_owned() => "p_solid_precipitation_6h",
        "wwZ6".to_owned() => "p_drizzle_6h",
        "wwMd".to_owned() => "p_fog_24h",
        "FX625".to_owned() => "p_gusts_25kn_6h",
        "FX640".to_owned() => "p_gusts_40kn_6h",
        "FX655".to_owned() => "p_gusts_55kn_6h",
        "wwDh".to_owned() => "p_straightform_precipitation_12h",
        "wwCh".to_owned() => "p_convective_precipitation_12h",
        "wwTh".to_owned() => "p_thunderstorms_12h",
        "wwPh".to_owned() => "p_precipitation_12h",
        "wwLh".to_owned() => "p_liquid_precipitation_12h",
        "wwFh".to_owned() => "p_freezing_rain_12h",
        "wwSh".to_owned() => "p_solid_precipitation_12h",
        "wwZh".to_owned() => "p_drizzle_12h",
        "R610".to_owned() => "p_precipitation_1mm_6h",
        "RRh".to_owned() => "precipitation_12h",
        "RRhc".to_owned() => "precipitation_12h_significant_weather",
        "ww3".to_owned() => "significant_weather_3h",
        "RRL1c".to_owned() => "liquid_precipitation_1h_significant_weather",
        "Rd00".to_owned() => "p_precipitation_00_24h",
        "Rd10".to_owned() => "p_precipitation_1mm_24h",
        "RRd".to_owned() => "precipitation_24h",
        "RRdc".to_owned() => "precipitation_24h_significant_weather",
        "Nlm".to_owned() => "cloud_cover_low_mid_7km",
        "wwPd".to_owned() => "p_precipitation_24h",
        "H_BsC".to_owned() => "cloud_base_convective_clouds",
        "wwTd".to_owned() => "p_thunderstorms_24h",
        "E_PPP".to_owned() => "e_surface_pressure",
        "SunD3".to_owned() => "sunshine_duration_3h",
        "WPc11".to_owned() => "opt_significant_weather_1h",
        "WPc31".to_owned() => "opt_significant_weather_3h",
        "WPc61".to_owned() => "opt_significant_weather_6h",
        "WPch1".to_owned() => "opt_significant_weather_12h",
        "WPcd1".to_owned() => "opt_significant_weather_24h",
        "Sa3".to_owned() => "accumulated_snow_3h",
        "Sa6".to_owned() => "accumulated_snow_6h",
        "Sah".to_owned() => "accumulated_snow_12h",
        "Sad".to_owned() => "accumulated_snow_24h",
        "Sa605".to_owned() => "p_snow_5cm_6h",
        "Sa610".to_owned() => "p_snow_10cm_6h",
        "Sa620".to_owned() => "p_snow_20cm_6h",
        "Sah05".to_owned() => "p_snow_5cm_12h",
        "Sah10".to_owned() => "p_snow_10cm_12h",
        "Sah30".to_owned() => "p_snow_30cm_12h",
        "Sad10".to_owned() => "p_snow_10cm_24h",
        "Sad30".to_owned() => "p_snow_30cm_24h",
        "Sad50".to_owned() => "p_snow_50cm_24h",
        "SnCv".to_owned() => "snow_depth"
    };
}