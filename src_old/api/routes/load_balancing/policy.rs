use actix_web::{get, web, Responder};
use crate::api::structs::*;

/// Handles GET requests for load balancing policy information.
///
/// This endpoint provides details about the current load balancing policy.
#[get("/load-balancing/policy")]
async fn load_balancing_policy() -> impl Responder {
    let policy = LoadBalancingPolicy {
        region_size: 1500,
        shard_threshold: 150,
        max_players_per_server: 1200,
        server_spawn_threshold: 75,
    };
    web::Json(policy)
}
