use actix_web::{get, web, Responder};
use crate::api::structs::*;
use rand::Rng;

/// Handles GET requests for cluster usage data.
///
/// This endpoint provides CPU and memory usage data for the cluster over time.
#[get("/clusters/usage")]
pub async fn cluster_usage() -> impl Responder {
    let usage = ClusterUsage {
        labels: vec!["00:00", "02:00", "04:00", "06:00", "08:00", "10:00", "12:00", "14:00", "16:00", "18:00", "20:00", "22:00"]
            .into_iter().map(String::from).collect(),
        cpu_usage: generate_random_data(50.0, 90.0, 12),
        memory_usage: generate_random_data(40.0, 80.0, 12),
    };
    web::Json(usage)
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
