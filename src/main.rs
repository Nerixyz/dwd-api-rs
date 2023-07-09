mod errors;
mod kml;
mod mosmix_cfg;
mod weather_forecast;
mod weather_report;

use crate::{
    mosmix_cfg::get_mosmix_stations, weather_forecast::get_forecast,
    weather_report::get_weather_report,
};
use actix_web::{get, http::header, middleware, web, App, HttpResponse, HttpServer};
use errors::DwdError;

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
                    .add(("X-DWD-API-Version", env!("CARGO_PKG_VERSION")))
                    // allow everyone to use this API
                    .add(("Access-Control-Allow-Origin", "*")),
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
