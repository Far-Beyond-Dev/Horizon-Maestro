use actix_web::{get, web, Responder};
use crate::api::structs::*;


/// Handles GET requests for subsystem information.
///
/// This endpoint provides details about various subsystems and their configurations.
#[get("/subsystems")]
async fn subsystems() -> impl Responder {
    let subsystems = vec![
        Subsystem {
            name: "Authentication".to_string(),
            enabled: true,
            icon: "🔐".to_string(),
            config: vec![
                SubsystemConfig { name: "API Key".to_string(), config_type: "text".to_string(), options: None },
                SubsystemConfig { name: "OAuth Settings".to_string(), config_type: "textarea".to_string(), options: None },
                SubsystemConfig { name: "Token Expiry".to_string(), config_type: "number".to_string(), options: None },
            ],
        },
        Subsystem {
            name: "Database".to_string(),
            enabled: true,
            icon: "💾".to_string(),
            config: vec![
                SubsystemConfig { name: "Connection String".to_string(), config_type: "text".to_string(), options: None },
                SubsystemConfig { name: "Pool Size".to_string(), config_type: "number".to_string(), options: None },
                SubsystemConfig { name: "Timeout".to_string(), config_type: "number".to_string(), options: None },
            ],
        },
        // Add more subsystems as needed
    ];
    web::Json(subsystems)
}