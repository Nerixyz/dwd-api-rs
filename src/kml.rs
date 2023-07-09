use crate::weather_forecast::{Forecast, ForecastReferenceModel};
use chrono::DateTime;
use serde::Deserialize;
use serde_json::Value;
use std::{collections::HashMap, error::Error, str::FromStr};

#[derive(Deserialize, Debug)]
struct Kml {
    #[serde(rename = "Document")]
    document: KmlDocument,
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
    #[serde(rename = "ProductDefinition")]
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
    forecasts: Vec<DwdForecast>,
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
    reference_time: String,
}

#[derive(Deserialize, Debug)]
struct ReferencedModel {
    #[serde(rename = "Model", default)]
    models: Vec<DwdModel>,
}

#[derive(Deserialize, Debug)]
struct ForecastTimeSteps {
    #[serde(rename = "TimeStep", default)]
    time_steps: Vec<String>,
}

#[derive(Deserialize, Debug)]
struct ProductDefinition {
    #[serde(rename = "Issuer")]
    issuer: String,
    #[serde(rename = "GeneratingProcess")]
    generating_process: String,
    #[serde(rename = "IssueTime")]
    issue_time: String,
    #[serde(rename = "ReferencedModel")]
    referenced_models: ReferencedModel,
    #[serde(rename = "ForecastTimeSteps")]
    forecast_time_steps: ForecastTimeSteps,
    // unused:
    //   ProductID:
    //     product_id: String,
    //   FormatCfg:
    //     format_config: { DefaultUndefSign: String },
}

type KmlForecastData = (HashMap<&'static str, Vec<Value>>, usize);

pub fn deserialize_to_forecast<R: std::io::Read>(raw: R) -> Result<Forecast, Box<dyn Error>> {
    let deserialized: Kml = serde_xml_rs::from_reader(raw)?;
    let product_def = deserialized.document.extended_data.product_definition;

    let (data, n_data_points) = kml_to_forecast_data(
        &deserialized.document.placemark.extended_data.forecasts,
        &product_def.forecast_time_steps.time_steps,
    )?;

    Ok(Forecast {
        issuer: product_def.issuer,
        name: deserialized.document.placemark.name,
        coordinates: deserialized.document.placemark.point.coordinates,
        description: deserialized.document.placemark.description,
        generating_process: product_def.generating_process,
        issue_time: DateTime::parse_from_rfc3339(&product_def.issue_time)?.timestamp_millis()
            as u64,
        reference_models: product_def
            .referenced_models
            .models
            .iter()
            .map(|m| ForecastReferenceModel {
                name: m.name.clone(),
                reference_time: DateTime::parse_from_rfc3339(&m.reference_time.clone())
                    .map(|d| d.timestamp_millis())
                    .unwrap_or(0) as u64,
            })
            .collect(),
        data,
        n_data_points,
    })
}

fn kml_to_forecast_data(
    forecasts: &[DwdForecast],
    time_steps: &[String],
) -> Result<KmlForecastData, Box<dyn Error>> {
    let mut json = HashMap::<&'static str, Vec<Value>>::new();
    let time_steps: Vec<Value> = time_steps
        .iter()
        .map(|s| {
            Value::from(
                DateTime::parse_from_rfc3339(s)
                    .map(|d| d.timestamp_millis())
                    .unwrap_or(0),
            )
        })
        .collect();
    let n_time_steps = time_steps.len();
    json.insert("time_steps", time_steps);

    for forecast in forecasts {
        let values: Vec<Value> = forecast
            .value
            .split_whitespace()
            .map(|s| f64::from_str(s).map(Value::from).unwrap_or(Value::Null))
            .collect();
        if values.len() != n_time_steps {
            continue;
        }
        if let Some(key) = KML_ELEMENT_TO_JSON_KEY.get(forecast.element_name.as_str()) {
            json.insert(key, values);
        }
    }

    Ok((json, n_time_steps))
}

static KML_ELEMENT_TO_JSON_KEY: phf::Map<&'static str, &'static str> = phf::phf_map! {
    "TTT" => "temp",
    "Td" => "dew_point",
    "TX" => "max_temp",
    "TN" => "min_temp",
    "DD" => "wind_direction",
    "FF" => "wind_speed",
    "FX1" => "max_wind_gust_1h",
    "FX3" => "max_wind_gust_3h",
    "FXh" => "max_wind_gust_12h",
    "RR1c" => "precipitation_1h_significant_weather",
    "RR1" => "precipitation_1h",
    "RR3c" => "precipitation_3h_significant_weather",
    "RR3" => "precipitation_3h",
    "RRS1c" => "snow_rain_eq_1h",
    "RRS3c" => "snow_rain_eq_3h",
    "ww" => "significant_weather",
    "W1W2" => "past_weather_6h",
    "N" => "total_cloud_cover",
    "Neff" => "effective_cloud_cover",
    "N05" => "cloud_cover_500ft",
    "Nl" => "low_cloud_cover",
    "Nm" => "midlevel_cloud_cover",
    "Nh" => "high_cloud_cover",
    "PPPP" => "surface_pressure",
    "T5cm" => "temp_5cm",
    "RadS3" => "shortwave_radiation_3h",
    "Rad1h" => "global_irradiance",
    "RadL3" => "longwave_radiation_3h",
    "VV" => "visibility",
    "SunD1" => "sunshine_last_hour",
    "FXh25" => "p_wind_gust_25kn_12h",
    "FXh40" => "p_wind_gust_40kn_12h",
    "FXh55" => "p_wind_gust_55kn_12h",
    "wwM" => "p_fog_1h",
    "wwM6" => "p_fog_6h",
    "wwMh" => "p_fog_12h",
    "Rh00" => "p_precipitation_0mm_12h",
    "R602" => "p_precipitation_p2mm_6h",
    "Rh02" => "p_precipitation_p2mm_12h",
    "Rd02" => "p_precipitation_p2mm_24h",
    "Rh10" => "p_precipitation_1mm_12h",
    "R650" => "p_precipitation_5mm_6h",
    "Rh50" => "p_precipitation_5mm_12h",
    "Rd50" => "p_precipitation_5mm_24h",
    "TG" => "min_temp_5cm_12h",
    "TM" => "mean_temp_24h",
    "DRR1" => "precipitation_duration_1h",
    "wwZ" => "p_drizzle_1h",
    "wwD" => "p_straitform_precipitation_1h",
    "wwC" => "p_convective_precipitation_1h",
    "wwT" => "p_thunderstorms_1h",
    "wwL" => "p_liquid_precipitation_1h",
    "wwS" => "p_solid_precipitation_1h",
    "wwF" => "p_freezing_rain_1h",
    "wwP" => "p_precipitation_1h",
    "VV10" => "p_visibility_below_1km",
    "E_TTT" => "e_temp",
    "E_FF" => "e_wind_speed",
    "E_DD" => "e_wind_direction",
    "E_Td" => "e_dew_point",
    "RR6" => "precipitation_6h",
    "RR6c" => "precipitation_6h_significant_weather",
    "R600" => "p_precipitation_0mm_6h",
    "R101" => "p_precipitation_p1mm_1h",
    "R102" => "p_precipitation_p2mm_1h",
    "R103" => "p_precipitation_p3mm_1h",
    "R105" => "p_precipitation_p5mm_1h",
    "R107" => "p_precipitation_p7mm_1h",
    "R110" => "p_precipitation_1mm_1h",
    "R120" => "p_precipitation_2mm_1h",
    "SunD" => "sunshine_duration_yesterday",
    "RSunD" => "rel_sunshine_duration_24h",
    "PSd00" => "p_rel_sunshine_duration_24h",
    "PSd30" => "p_rel_sunshine_duration_30p_24h",
    "PSd60" => "p_rel_sunshine_duration_60p_24h",
    "RRad1" => "global_irradiance_1h",
    "PEvap" => "potential_evapotranspiration_24h",
    "R130" => "p_precipitation_3mm_1h",
    "R150" => "p_precipitation_5mm_1h",
    "RR1o1" => "p_precipitation_10mm_1h",
    "RR1w1" => "p_precipitation_15mm_1h",
    "RR1u1" => "p_precipitation_25mm_1h",
    "wwD6" => "p_straightform_precipitation_6h",
    "wwC6" => "p_convective_precipitation_6h",
    "wwT6" => "p_thunderstorms_6h",
    "wwP6" => "p_precipitation_6h",
    "wwL6" => "p_liquid_precipitation_6h",
    "wwF6" => "p_freezing_rain_6h",
    "wwS6" => "p_solid_precipitation_6h",
    "wwZ6" => "p_drizzle_6h",
    "wwMd" => "p_fog_24h",
    "FX625" => "p_gusts_25kn_6h",
    "FX640" => "p_gusts_40kn_6h",
    "FX655" => "p_gusts_55kn_6h",
    "wwDh" => "p_straightform_precipitation_12h",
    "wwCh" => "p_convective_precipitation_12h",
    "wwTh" => "p_thunderstorms_12h",
    "wwPh" => "p_precipitation_12h",
    "wwLh" => "p_liquid_precipitation_12h",
    "wwFh" => "p_freezing_rain_12h",
    "wwSh" => "p_solid_precipitation_12h",
    "wwZh" => "p_drizzle_12h",
    "R610" => "p_precipitation_1mm_6h",
    "RRh" => "precipitation_12h",
    "RRhc" => "precipitation_12h_significant_weather",
    "ww3" => "significant_weather_3h",
    "RRL1c" => "liquid_precipitation_1h_significant_weather",
    "Rd00" => "p_precipitation_00_24h",
    "Rd10" => "p_precipitation_1mm_24h",
    "RRd" => "precipitation_24h",
    "RRdc" => "precipitation_24h_significant_weather",
    "Nlm" => "cloud_cover_low_mid_7km",
    "wwPd" => "p_precipitation_24h",
    "H_BsC" => "cloud_base_convective_clouds",
    "wwTd" => "p_thunderstorms_24h",
    "E_PPP" => "e_surface_pressure",
    "SunD3" => "sunshine_duration_3h",
    "WPc11" => "opt_significant_weather_1h",
    "WPc31" => "opt_significant_weather_3h",
    "WPc61" => "opt_significant_weather_6h",
    "WPch1" => "opt_significant_weather_12h",
    "WPcd1" => "opt_significant_weather_24h",
    "Sa3" => "accumulated_snow_3h",
    "Sa6" => "accumulated_snow_6h",
    "Sah" => "accumulated_snow_12h",
    "Sad" => "accumulated_snow_24h",
    "Sa605" => "p_snow_5cm_6h",
    "Sa610" => "p_snow_10cm_6h",
    "Sa620" => "p_snow_20cm_6h",
    "Sah05" => "p_snow_5cm_12h",
    "Sah10" => "p_snow_10cm_12h",
    "Sah30" => "p_snow_30cm_12h",
    "Sad10" => "p_snow_10cm_24h",
    "Sad30" => "p_snow_30cm_24h",
    "Sad50" => "p_snow_50cm_24h",
    "SnCv" => "snow_depth"
};
