use actix_web::{get, web, Responder};
use crate::api::structs::*;

/// Handles GET requests for database information.
///
/// This endpoint provides details about the database instances used by the system.
#[get("/databases")]
async fn databases() -> impl Responder {
    let databases = vec![
        DatabaseInfo {
            name: "Test Deploy Database".to_string(),
            region: "US-East".to_string(),
            size: "2.7 GB".to_string(),
            db_type: "Graph".to_string(),
            address: "test-deploy-database.myproject.creator.example.com".to_string(),
            queries_per_second: 41,
            active_connections: 537,
        },
        // Add more database entries as needed
    ];
    web::Json(databases)
}
