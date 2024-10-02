use actix_web::{get, web, Responder};
use crate::api::structs::*;

/// Handles GET requests for network bandwidth usage.
///
/// This endpoint provides overall bandwidth usage statistics.
#[get("/network/bandwidth")]
async fn network_bandwidth() -> impl Responder {
    let bandwidth = BandwidthUsage {
        total_bandwidth: 400,
        total_bandwidth_change: 14.3,
        incoming_bandwidth: 180,
        incoming_bandwidth_change: 5.7,
        outgoing_bandwidth: 220,
        outgoing_bandwidth_change: 8.6,
    };
    web::Json(bandwidth)
}