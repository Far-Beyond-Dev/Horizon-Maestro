use actix_web::{get, web, Responder};
use crate::api::structs::*;

/// Handles GET requests for update history.
///
/// This endpoint provides a history of past system updates.
#[get("/maintenance/updates/history")]
async fn update_history() -> impl Responder {
    let history = vec![
        UpdateHistory { id: 1, name: "Hotfix".to_string(), version: "1.2.4".to_string(), date: "2024-08-01".to_string(), status: "Successful".to_string() },
        UpdateHistory { id: 2, name: "Performance Update".to_string(), version: "1.9.2".to_string(), date: "2024-07-15".to_string(), status: "Successful".to_string() },
        UpdateHistory { id: 3, name: "Security Update".to_string(), version: "1.2.3".to_string(), date: "2024-07-01".to_string(), status: "Failed".to_string() },
    ];
    web::Json(history)
}


/// Handles GET requests for system update information.
///
/// This endpoint provides details about available system updates.
#[get("/maintenance/updates/avalible")]
async fn avalible_updates() -> impl Responder {
    let updates = vec![
        UpdateInfo { id: 1, name: "Security Patch".to_string(), version: "1.2.5".to_string(), size: "25 MB".to_string(), importance: "Critical".to_string() },
        UpdateInfo { id: 2, name: "Feature Update".to_string(), version: "2.0.0".to_string(), size: "150 MB".to_string(), importance: "Recommended".to_string() },
        UpdateInfo { id: 3, name: "Bug Fixes".to_string(), version: "1.9.3".to_string(), size: "10 MB".to_string(), importance: "Optional".to_string() },
    ];
    web::Json(updates)
}