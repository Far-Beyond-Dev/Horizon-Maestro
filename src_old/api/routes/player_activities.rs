use actix_web::{get, web, Responder};
use crate::api::structs::*;

/// Handles GET requests for player activities.
///
/// This endpoint provides a list of recent player activities.
#[get("/player-activities")]
async fn player_activities() -> impl Responder {
    let activities = vec![
        PlayerActivity {
            player: "Alice".to_string(),
            action: "Joined server US-East".to_string(),
            time: "2 minutes ago".to_string(),
            avatar: "https://flowbite.com/docs/images/people/profile-picture-5.jpg".to_string(),
        },
        PlayerActivity {
            player: "Bob".to_string(),
            action: "Purchased premium package".to_string(),
            time: "15 minutes ago".to_string(),
            avatar: "https://flowbite.com/docs/images/people/profile-picture-2.jpg".to_string(),
        },
        // Add more activities as needed
    ];
    web::Json(activities)
}