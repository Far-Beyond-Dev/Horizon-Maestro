use actix_web::{get, web, Responder};
use crate::api::structs::*;
use chrono::Utc;

/// Handles GET requests for the system audit log.
///
/// This endpoint provides entries from the system audit log.
#[get("/security/audit-log")]
async fn audit_log() -> impl Responder {
    let logs = vec![
        AuditLog { id: 1, timestamp: Utc::now(), user: "Alice".to_string(), action: "Create".to_string(), resource: "New Server Instance".to_string(), details: "Created server instance 'US-West-01'".to_string() },
        AuditLog { id: 2, timestamp: Utc::now(), user: "Bob".to_string(), action: "Edit".to_string(), resource: "Load Balancing Policy".to_string(), details: "Updated region size from 1000m to 1500m".to_string() },
        AuditLog { id: 3, timestamp: Utc::now(), user: "Charlie".to_string(), action: "Remove".to_string(), resource: "User Account".to_string(), details: "Removed user 'inactive_user_123'".to_string() },
    ];
    web::Json(logs)
}
