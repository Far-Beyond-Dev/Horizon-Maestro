use actix_web::{get, web, Responder};
use crate::api::structs::*;
use rand::Rng;

/// Handles GET requests for network latency information.
///
/// This endpoint provides detailed statistics about network latency.
#[get("/network/latency")]
async fn network_latency() -> impl Responder {
    let latency = NetworkLatency {
        avg_latency: 45,
        peak_latency: 120,
        packet_loss: 0.5,
        latency_over_time: generate_random_data(30, 100, 24),
        avg_latency_change: -2.5,
        peak_latency_change: 15.0,
        packet_loss_change: -0.1,
        peak_latency_trend: generate_random_data(80, 150, 10),
        latency_distribution: [
            ("0-50ms".to_string(), 45),
            ("51-100ms".to_string(), 30),
            ("101-150ms".to_string(), 15),
            ("151-200ms".to_string(), 7),
            ("200ms+".to_string(), 3),
        ].iter().cloned().collect(),
    };
    web::Json(latency)
}

/// Generates random data within a specified range.
///
/// This helper function is used to create mock data for various metrics.
///
/// # Arguments
///
/// * `min` - The minimum value of the range (inclusive)
/// * `max` - The maximum value of the range (inclusive)
/// * `count` - The number of random values to generate
///
/// # Returns
///
/// A vector of randomly generated values within the specified range.
fn generate_random_data<T>(min: T, max: T, count: usize) -> Vec<T>
where
    T: rand::distributions::uniform::SampleUniform + Copy + PartialOrd,
{
    let mut rng = rand::thread_rng();
    (0..count).map(|_| rng.gen_range(min..=max)).collect()
}

// Routes
