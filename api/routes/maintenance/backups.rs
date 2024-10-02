use actix_web::{get, web, Responder};
use crate::api::structs::*;

/// Handles GET requests for system backup information.
///
/// This endpoint provides details about system backups.
#[get("/maintenance/backups")]
async fn backups() -> impl Responder {
    let backups = vec![
        Backup { id: 1, name: "Full Backup".to_string(), date: "2024-08-09 14:30".to_string(), size: "2.3 GB".to_string(), status: "Completed".to_string() },
        Backup { id: 2, name: "Incremental Backup".to_string(), date: "2024-08-08 22:00".to_string(), size: "500 MB".to_string(), status: "Completed".to_string() },
        Backup { id: 3, name: "Database Backup".to_string(), date: "2024-08-07 03:00".to_string(), size: "1.1 GB".to_string(), status: "Completed".to_string() },
    ];
    web::Json(backups)
}
