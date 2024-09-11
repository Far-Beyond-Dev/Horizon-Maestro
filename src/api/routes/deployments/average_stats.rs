use actix_web::{get, web, Responder};
use serde::Serialize;

#[derive(Serialize)]
struct PlayerStats {
    active_players: u32,
    max_player_count: u32,
    new_players: u32,
    total_players: u32,
    retention_rate: f32,
    retention_rate_change: f32,
}

/// Handles GET requests for player statistics.
///
/// This endpoint provides details about the current player activity and retention.
#[get("/deployments/stats")]
async fn player_stats() -> impl Responder {
    let stats = PlayerStats {
        active_players: 5200,
        max_player_count: 6000,
        new_players: 700,
        total_players: 50000,
        retention_rate: 85.0,
        retention_rate_change: 2.5,
    };

    web::Json(stats)
}