mod kml;
mod mosmix_cfg;
mod weather_forecast;
mod weather_report;

use crate::{
    mosmix_cfg::get_mosmix_stations, weather_forecast::get_forecast,
    weather_report::get_weather_report,
};
use actix_web::{
    error, get,
    http::{header, StatusCode},
    middleware, web, App, HttpResponse, HttpServer,
};
use derive_more::{Display, Error};
use serde_json::json;

#[derive(Debug, Display, Error)]
pub enum DwdError {
    #[display(fmt = "not found")]
    NotFound,

    #[display(fmt = "invalid file")]
    InvalidFile,

    #[display(fmt = "read kml error")]
    ReadKmlError,
}

impl error::ResponseError for DwdError {
    fn status_code(&self) -> StatusCode {
        match *self {
            DwdError::NotFound => StatusCode::NOT_FOUND,
            DwdError::InvalidFile => StatusCode::INTERNAL_SERVER_ERROR,
            DwdError::ReadKmlError => StatusCode::INTERNAL_SERVER_ERROR,
        }
    }

    fn error_response(&self) -> HttpResponse {
        HttpResponse::build(self.status_code()).json(json!({
            "error": self.to_string()
        }))
    }
}

#[get("/forecast/{station}")]
async fn handle_station(station: web::Path<String>) -> Result<HttpResponse, DwdError> {
    let forecast = get_forecast(&station).await?;
    Ok(HttpResponse::Ok()
        .insert_header((header::CACHE_CONTROL, "max-age=1000"))
        .json(forecast))
}

#[get("/stations")]
async fn handle_get_stations() -> Result<HttpResponse, DwdError> {
    let stations = get_mosmix_stations().await?;
    Ok(HttpResponse::Ok()
        .insert_header((header::CACHE_CONTROL, "max-age=604800"))
        .json(stations))
}

#[get("/report/{station}")]
async fn handle_get_report(station: web::Path<String>) -> Result<HttpResponse, DwdError> {
    let report = get_weather_report(station.into_inner()).await?;
    Ok(HttpResponse::Ok().json(report))
}

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    dotenv::dotenv().expect("No .env file");
    HttpServer::new(|| {
        App::new()
            .wrap(middleware::Compress::default())
            .wrap(
                middleware::DefaultHeaders::new()
                    .header("X-DWD-API-Version", env!("CARGO_PKG_VERSION"))
                    // allow everyone to use this API
                    .header("Access-Control-Allow-Origin", "*"),
            )
            .service(handle_station)
            .service(handle_get_stations)
            .service(handle_get_report)
    })
    .bind(format!(
        "{}:{}",
        std::env::var("DWD_API_HOST").unwrap_or("localhost".to_owned()),
        std::env::var("DWD_API_PORT").unwrap_or("8080".to_owned())
    ))?
    .run()
    .await
}
