use actix_web::{get, web, Responder};
use crate::api::structs::*;

/// Handles GET requests for deployment information.
///
/// This endpoint provides details about current game deployments.
#[get("/deployments")]
async fn deployments() -> impl Responder {
    let deployments = vec![
        DeploymentInfo {
            name: "Test Deploy".to_string(),
            region: "US-East".to_string(),
            avg_load: 2.7931,
            avg_latency: "20ms".to_string(),
            inbound_traffic: "47.3 mb/s".to_string(),
            outbound_traffic: "56.2 mb/s".to_string(),
            players: 555,
            status: "Online".to_string(),
        },
        // Add more deployments as needed
    ];
    web::Json(deployments)
}