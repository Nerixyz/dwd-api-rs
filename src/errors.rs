#[derive(Debug, thiserror::Error, actix_web_error::Json)]
pub enum DwdError {
    // weather report
    #[error("No report was found for this station")]
    #[status(404)]
    NoReport,
    #[error("The report's CSV file didn't have a header row")]
    #[status(500)]
    NoHeaderRow,
    #[error("The report's CSV file didn't have a unit row")]
    #[status(500)]
    NoUnitRow,
    #[error("The report's CSV didn't declare the exact units for each property or too many")]
    #[status(500)]
    UnitMismatch,
    #[error("The report's CSV file contained an invalid row")]
    #[status(500)]
    BadCsvLine,

    // weather forecast
    #[error("No forecast was found for this station")]
    #[status(404)]
    NoForecast,
    #[error("The forecast's zip file was invalid")]
    #[status(500)]
    BadZipFile,
    #[error("The forecast's zip file didn't contain a forecast")]
    #[status(500)]
    NoZipEntry,
    #[error("Couldn't read KML file ({0})")]
    #[status(500)]
    InvalidKml(serde_xml_rs::Error),
    #[error("Couldn't parse issue-time ({0})")]
    #[status(500)]
    InvalidIssueTime(chrono::ParseError),

    // stations
    #[error("No station listing was found")]
    #[status(404)]
    NoStationListing,

    // generic
    #[error("Internal error")]
    #[status(500)]
    InternalError,
}
