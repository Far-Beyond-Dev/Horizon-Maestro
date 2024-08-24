use actix_web::{get, web, Responder};
use crate::api::structs::*;
use rand::Rng;
use chrono::Utc;

/// Handles GET requests for network connection health.
///
/// This endpoint provides health status information for server connections.
#[get("/network/health")]
async fn connection_health() -> impl Responder {
    let health: Vec<ConnectionHealth> = (1..=10).map(|i| ConnectionHealth {
        server: format!("Server {}", i),
        healthy: rand::thread_rng().gen_bool(0.8),
        ping: rand::thread_rng().gen_range(10..110),
        last_checked: Utc::now().to_rfc3339(),
    }).collect();
    web::Json(health)
}
