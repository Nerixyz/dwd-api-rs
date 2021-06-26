mod kml;
mod weather_forecast;
mod mosmix_cfg;
mod weather_report;

use actix_web::{HttpServer, get, App, web, error, HttpResponse, middleware};
use actix_web::dev::HttpResponseBuilder;
use actix_web::http::{StatusCode, header};
use derive_more::{Error, Display};
use serde_json::json;
use crate::weather_forecast::get_forecast;
use crate::mosmix_cfg::get_mosmix_stations;
use crate::weather_report::get_weather_report;

#[derive(Debug, Display, Error)]
pub enum DwdError {
    #[display(fmt="not found")]
    NotFound,

    #[display(fmt="invalid file")]
    InvalidFile,

    #[display(fmt="read kml error")]
    ReadKmlError,
}

impl error::ResponseError for DwdError {
    fn error_response(&self) -> HttpResponse {
        HttpResponseBuilder::new(self.status_code())
            .json(json!({
                "error": self.to_string()
            }))
    }

    fn status_code(&self) -> StatusCode {
        match *self {
            DwdError::NotFound => StatusCode::NOT_FOUND,
            DwdError::InvalidFile => StatusCode::INTERNAL_SERVER_ERROR,
            DwdError::ReadKmlError => StatusCode::INTERNAL_SERVER_ERROR,
        }
    }
}

#[get("/forecast/{station}")]
async fn handle_station(web::Path(station): web::Path<String>) -> Result<HttpResponse, DwdError> {
    let forecast = get_forecast(&station).await?;
    Ok(HttpResponseBuilder::new(StatusCode::OK)
        .header(header::CACHE_CONTROL, "max-age=1000")
        .json(forecast))
}

#[get("/stations")]
async fn handle_get_stations() -> Result<HttpResponse, DwdError> {
    let stations = get_mosmix_stations().await?;
    Ok(HttpResponseBuilder::new(StatusCode::OK)
        .header(header::CACHE_CONTROL, "max-age=604800")
        .json(stations))
}

#[get("/report/{station}")]
async fn handle_get_report(web::Path(station): web::Path<String>) -> Result<HttpResponse, DwdError> {
    let report = get_weather_report(station).await?;
    Ok(HttpResponseBuilder::new(StatusCode::OK)
        .json(report))
}

#[actix_web::main]
async fn main() ->  std::io::Result<()> {
    dotenv::dotenv().expect("No .env file");
    HttpServer::new(|| {
        App::new()
            .wrap(middleware::Compress::default())
            .wrap(middleware::DefaultHeaders::new()
                .header("X-DWD-API-Version", env!("CARGO_PKG_VERSION"))
                // allow everyone to use this API
                .header("Access-Control-Allow-Origin", "*")
            )
            .service(handle_station)
            .service(handle_get_stations)
            .service(handle_get_report)
    })
        .bind(
            format!(
                "{}:{}",
                std::env::var("DWD_API_HOST").unwrap_or("localhost".to_owned()),
                std::env::var("DWD_API_PORT").unwrap_or("8080".to_owned())))?
        .run()
        .await
}
