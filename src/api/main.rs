use actix_web::{web, App, HttpServer};
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
    println!("Hello from the API!");



    // Create and configure the HTTP server
    
    /*
     * Oh, hello there, future me (or some poor soul unfortunate enough to maintain this code).
     * Behold, the Leaning Tower of Services, a testament to my ability to stack things vertically
     * until they reach the stratosphere. Why use a simple list when you can create a skyscraper
     * of services that makes the Empire State Building look like a garden gnome?
     *
     * Legend has it that for every .service() call, a developer loses a small piece of their sanity.
     * By that metric, I must be absolutely bonkers by now. But who needs sanity when you have
     * a codebase that doubles as a vertigo simulator?
     *
     * I'd like to formally apologize to my keyboard's 'dot' key and 'v' key (for all those Ctrl+V presses).
     * They didn't deserve this abuse. They were good keys, always there for me, and this is how I repay them.
     *
     * Some say brevity is the soul of wit. Well, clearly I decided to go for the "verbose and witless" approach.
     * Because why communicate clearly when you can create a wall of text that rivals ancient Egyptian hieroglyphics
     * in its inscrutability?
     *
     * If you're reading this and thinking, "There must be a better way," congratulations! 
     * You're absolutely correct. There is. But where's the fun in that? Why solve a problem efficiently
     * when you can create a monument to inefficiency that future generations will study with awe and confusion?
     *
     * So, dear reader, as you embark on the Herculean task of deciphering this code,
     * remember: it's not a bug, it's a feature. A feature designed to test the limits
     * of human patience and scrolling endurance.
     *
     * May the force be with you. You're gonna need it.
     */

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




    ////////////////////////////////////////////////////////////////////////
    //                                 WIP                                //
    ////////////////////////////////////////////////////////////////////////
    //
    //  macro_rules! add_services {
    //      ($app:expr, $($path:expr => $handler:expr),* $(,)?) => {
    //          $($app.service(web::scope($path).service($handler)))*
    //      };
    //  }
    //  
    //  let server = HttpServer::new(move || {
    //      let app = App::new().app_data(pool_data.clone());
    //  
    //      add_services!(app,
    //          "/network" => routes::network::clusters::cluster_usage,
    //          "/network" => routes::network::latency::network_latency,
    //          "/network" => routes::network::regions::network_regions,
    //          "/network" => routes::network::bandwidth::network_bandwidth,
    //          "/network" => routes::network::cluster_bandwidth::cluster_bandwidth,
    //          "/network" => routes::network::server_bandwidth::server_bandwidth,
    //          "/network" => routes::network::health::connection_health,
    //          "/servers" => routes::servers::get_servers,
    //          "/player-activities" => routes::player_activities::player_activities,
    //          "/deployments" => routes::deployments::deployments,
    //          "/databases" => routes::databases::databases,
    //          "/alerts" => routes::alerts::alerts,
    //          "/maintenance/updates" => routes::maintenance::updates::avalible_updates,
    //          "/maintenance/updates" => routes::maintenance::updates::update_history,
    //          "/maintenance/tasks" => routes::maintenance::tasks::scheduled_tasks,
    //          "/maintenance/tasks" => routes::maintenance::tasks::task_history,
    //          "/maintenance/backups" => routes::maintenance::backups::backups,
    //          "/load-balancing" => routes::load_balancing::policy::load_balancing_policy,
    //          "/security/access" => routes::security::access::user_access,
    //          "/security/audit" => routes::security::audit_log::audit_log,
    //          "/subsystems" => routes::subsystems::subsystems,
    //      )
    //  });



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