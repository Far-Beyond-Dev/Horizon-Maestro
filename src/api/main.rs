use actix_web:: {web, App, HttpServer};
use tokio::sync::oneshot;
use crate::api::setup_db::setup_db;
use fern::Dispatch;
use log::LevelFilter;
use std::fs::File;
use crate::api::routes;

/// Sets up the logging system for the application.
///
/// This function configures logging to both stdout and a file named "app.log".
///
/// # Returns
///
/// * `Ok(())` if logging setup is successful
/// * `Err(fern::InitError)` if there's an error setting up logging
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
        .level(LevelFilter::Debug) // Adjust log level as needed
        .chain(std::io::stdout())
        .chain(log_file)
        .apply()?;

    Ok(())
}

/// Runs the API server.
///
/// This function sets up the database, configures logging, and starts the HTTP server
/// with all the defined routes. It also handles graceful shutdown when receiving a shutdown signal.
///
/// # Arguments
///
/// * `shutdown_rx` - A oneshot receiver for shutdown signals
///
/// # Returns
///
/// * `Ok(())` if the server runs and shuts down successfully
/// * `Err(std::io::Error)` if there's an error starting or running the server
pub async fn run_api_server(shutdown_rx: oneshot::Receiver<()>) -> std::io::Result<()> {
    // Set up the database connection pool
    let pool = setup_db().await;
    let pool_data = web::Data::new(pool);

    // Configure logging
    setup_logging().expect("Failed to set up logging");

    // Create and configure the HTTP server
    let server = HttpServer::new(move || {
        App::new()
            .app_data(pool_data.clone())
            .service(routes::network::clusters::cluster_usage)
            .service(routes::servers::get_servers)
            .service(routes::player_activities::player_activities)
            .service(routes::deployments::deployments)
            .service(routes::databases::databases)
            .service(routes::alerts::alerts)
            .service(routes::network::latency::network_latency)
            .service(routes::network::regions::network_regions)
            .service(routes::network::bandwidth::network_bandwidth)
            .service(routes::network::cluster_bandwidth::cluster_bandwidth)
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