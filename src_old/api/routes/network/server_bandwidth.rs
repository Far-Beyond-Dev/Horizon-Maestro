use actix_web::{get, web, Responder};
use crate::api::structs::*;
use rand::Rng;

/// Handles GET requests for server-specific bandwidth information.
///
/// This endpoint provides bandwidth usage data for individual servers.
#[get("/network/server-bandwidth")]
async fn server_bandwidth() -> impl Responder {
    let servers: Vec<ServerBandwidth> = (0..20).map(|i| ServerBandwidth {
        name: format!("Server {}", i + 1),
        cluster: format!("Cluster {}", (65 + (i % 5)) as u8 as char),
        bandwidth: rand::thread_rng().gen_range(10..60),
    }).collect();
    web::Json(servers)
}