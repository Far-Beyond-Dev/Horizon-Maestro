use actix_web::{get, web, Responder};
use crate::api::structs::*;

/// Handles GET requests for network region information.
///
/// This endpoint provides a list of network regions with their geographic and latency data.
#[get("/network/regions")]
async fn network_regions() -> impl Responder {
    let regions = vec![
        Region { name: "North America".to_string(), lat: 40.0, lon: -100.0, latency: 30 },
        Region { name: "Europe".to_string(), lat: 50.0, lon: 10.0, latency: 45 },
        Region { name: "Asia".to_string(), lat: 30.0, lon: 100.0, latency: 60 },
        Region { name: "South America".to_string(), lat: -20.0, lon: -60.0, latency: 55 },
        Region { name: "Australia".to_string(), lat: -25.0, lon: 135.0, latency: 70 },
    ];
    web::Json(regions)
}