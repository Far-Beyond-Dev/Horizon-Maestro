use actix_web::{get, web, Responder};
use crate::api::structs::*;

/// Handles GET requests for user access information.
///
/// This endpoint provides details about user access and permissions.
#[get("/security/access")]
async fn user_access() -> impl Responder {
    let users = vec![
        UserAccess { id: 1, name: "John Doe".to_string(), email: "john@example.com".to_string(), role: "Admin".to_string(), permissions: vec!["read".to_string(), "write".to_string(), "delete".to_string()] },
        UserAccess { id: 2, name: "Jane Smith".to_string(), email: "jane@example.com".to_string(), role: "Editor".to_string(), permissions: vec!["read".to_string(), "write".to_string()] },
        UserAccess { id: 3, name: "Bob Johnson".to_string(), email: "bob@example.com".to_string(), role: "Viewer".to_string(), permissions: vec!["read".to_string()] },
    ];
    web::Json(users)
}