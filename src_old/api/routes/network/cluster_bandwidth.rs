use actix_web::{get, web, Responder};
use crate::api::structs::*;
use rand::Rng;

/// Handles GET requests for cluster-specific bandwidth information.
///
/// This endpoint provides bandwidth usage data for individual clusters.
#[get("/network/cluster-bandwidth")]
async fn cluster_bandwidth() -> impl Responder {
    let clusters: Vec<ClusterBandwidth> = (0..10).map(|i| ClusterBandwidth {
        name: format!("Cluster {}", (65 + i) as u8 as char),
        bandwidth: rand::thread_rng().gen_range(50..250),
        change: rand::thread_rng().gen_range(-5.0..5.0),
    }).collect();
    web::Json(clusters)
}
