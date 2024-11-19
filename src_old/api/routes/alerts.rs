use actix_web::{get, web, Responder};
use crate::api::structs::*;
use chrono::Utc;

/// Handles GET requests for system alerts.
///
/// This endpoint provides a list of current system alerts or notifications.
#[get("/alerts")]
async fn alerts() -> impl Responder {
    let alerts = vec![
        AlertInfo {
            id: 1,
            clusters: vec!["Cluster A".to_string(), "Cluster B".to_string()],
            servers: vec!["Server 1".to_string(), "Server 3".to_string()],
            level: "ERROR".to_string(),
            description: "CPU usage high".to_string(),
            timestamp: Utc::now(),
        },
        // Add more alerts as needed
    ];
    web::Json(alerts)
}