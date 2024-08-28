use actix_web::{http::header, web::{self, route}, App, HttpServer};
use actix_cors::Cors;
use tokio::sync::oneshot;
use crate::api::setup_db::setup_db;
use fern::Dispatch;
use log::LevelFilter;
use std::fs::File;
use crate::api::routes;

/// Sets up the logging system for the application.
fn setup_logging() -> Result<(), fern::InitError> {
    let log_file = File::create("app.log")?;
    
    Dispatch::new()
        .format(|out, message, record| {
            out.finish(format_args!(
                "[{}][{}]: {}",
                chrono::Local::now().format("%Y-%m-%d %H:%M:%S"),
                record.level(),
                message
            ))
        })
        .level(LevelFilter::Debug)
        .chain(std::io::stdout())
        .chain(log_file)
        .apply()?;

    Ok(())
}

/// Runs the API server.
pub async fn run_api_server(shutdown_rx: oneshot::Receiver<()>) -> std::io::Result<()> {
    // Set up the database connection pool
    let pool = setup_db().await;
    let pool_data = web::Data::new(pool);

    // Configure logging
    setup_logging().expect("Failed to set up logging");
    println!("Hello from the API!");

    let server = HttpServer::new(move || {
        App::new()
            .app_data(pool_data.clone())
            .wrap(
                Cors::default()
                    .allow_any_origin() // Allow all origins
                    .allow_any_method()  // Allow any HTTP method
                    .allow_any_header()  // Allow any header
                    .supports_credentials() // Allow credentials
                    .max_age(3600), // Cache the CORS response for 1 hour
            )
            .service(routes::dashboard::systemAlerts::get_insights)
            .service(routes::deployments::averageStats::player_stats)
            .service(routes::deployments::playersByPlatform::players_by_platform)
            .service(routes::network::clusters::cluster_usage)
            .service(routes::servers::get_servers)
            .service(routes::player_activities::player_activities)
            .service(routes::deployments::list::deployments)
            .service(routes::databases::databases)
            .service(routes::alerts::alerts)
            .service(routes::network::latency::network_latency)
            .service(routes::network::regions::network_regions)
            .service(routes::network::bandwidth::network_bandwidth)
            .service(routes::network::cluster_bandwidth::cluster_bandwidth)
            .service(routes::network::cluster_usage::cluster_usage)
            .service(routes::network::server_bandwidth::server_bandwidth)
            .service(routes::network::health::connection_health)
            .service(routes::maintenance::updates::avalible_updates)
            .service(routes::maintenance::updates::update_history)
            .service(routes::maintenance::tasks::scheduled_tasks)
            .service(routes::maintenance::tasks::task_history)
            .service(routes::maintenance::backups::backups)
            .service(routes::load_balancing::policy::load_balancing_policy)
            .service(routes::security::access::user_access)
            .service(routes::security::audit_log::audit_log)
            .service(routes::subsystems::subsystems)
    })
    .bind("0.0.0.0:8080")?
    .run();

    println!("🗺️  API Server running on 0.0.0.0:8080");

    // Run the server and handle shutdown gracefully
    tokio::select! {
        _ = server => {
            println!("Server stopped unexpectedly");
        },
        _ = shutdown_rx => {
            println!("Shutting down API server");
        }
    }

    Ok(())
}
