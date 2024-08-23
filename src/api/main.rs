use actix_web::{get, web, App, HttpServer, Responder};
use serde::Serialize;
use chrono::{DateTime, Utc};
use rand::Rng;
use std::collections::HashMap;
use tokio::sync::oneshot;
use sqlx::sqlite::SqlitePool;
use crate::api::setup_db::setup_db;
use sqlx::Row;
use fern::Dispatch;
use log::LevelFilter;
use std::fs::File;

// Structs for various data types

/// Represents cluster usage data including CPU and memory usage over time.
#[derive(Serialize)]
struct ClusterUsage {
    /// Time labels for the usage data points
    labels: Vec<String>,
    /// CPU usage percentages corresponding to each label
    cpu_usage: Vec<f64>,
    /// Memory usage percentages corresponding to each label
    memory_usage: Vec<f64>,
}

/// Represents information about a game server.
#[derive(Serialize)]
struct Server {
    /// Name of the server
    name: String,
    /// Current status of the server (e.g., "Online", "Offline")
    status: String,
    /// Number of players currently on the server
    players: u32,
    /// Current CPU usage percentage
    cpu: f64,
    /// Current memory usage percentage
    memory: f64,
}

/// Represents a player activity event.
#[derive(Serialize)]
struct PlayerActivity {
    /// Name of the player
    player: String,
    /// Description of the action performed
    action: String,
    /// Time when the action occurred (in human-readable format)
    time: String,
    /// URL of the player's avatar image
    avatar: String,
}

/// Represents information about a game deployment.
#[derive(Serialize)]
struct DeploymentInfo {
    /// Name of the deployment
    name: String,
    /// Geographic region of the deployment
    region: String,
    /// Average server load
    avg_load: f64,
    /// Average latency (in milliseconds)
    avg_latency: String,
    /// Inbound network traffic
    inbound_traffic: String,
    /// Outbound network traffic
    outbound_traffic: String,
    /// Number of players in the deployment
    players: u32,
    /// Current status of the deployment
    status: String,
}

/// Represents information about a database instance.
#[derive(Serialize)]
struct DatabaseInfo {
    /// Name of the database
    name: String,
    /// Geographic region of the database
    region: String,
    /// Size of the database
    size: String,
    /// Type of the database (e.g., "Graph", "Relational")
    db_type: String,
    /// Network address of the database
    address: String,
    /// Number of queries processed per second
    queries_per_second: u32,
    /// Number of active connections to the database
    active_connections: u32,
}

/// Represents an alert or notification in the system.
#[derive(Serialize)]
struct AlertInfo {
    /// Unique identifier for the alert
    id: u32,
    /// List of affected clusters
    clusters: Vec<String>,
    /// List of affected servers
    servers: Vec<String>,
    /// Severity level of the alert
    level: String,
    /// Detailed description of the alert
    description: String,
    /// Timestamp when the alert was generated
    timestamp: DateTime<Utc>,
}

/// Represents network latency statistics.
#[derive(Serialize)]
struct NetworkLatency {
    /// Average latency in milliseconds
    avg_latency: u32,
    /// Peak latency in milliseconds
    peak_latency: u32,
    /// Packet loss percentage
    packet_loss: f32,
    /// Historical latency data over time
    latency_over_time: Vec<u32>,
    /// Change in average latency compared to previous period
    avg_latency_change: f32,
    /// Change in peak latency compared to previous period
    peak_latency_change: f32,
    /// Change in packet loss compared to previous period
    packet_loss_change: f32,
    /// Trend of peak latency over time
    peak_latency_trend: Vec<u32>,
    /// Distribution of latency across different ranges
    latency_distribution: HashMap<String, u32>,
}

/// Represents a geographic region with network statistics.
#[derive(Serialize)]
struct Region {
    /// Name of the region
    name: String,
    /// Latitude coordinate
    lat: f32,
    /// Longitude coordinate
    lon: f32,
    /// Average latency to this region in milliseconds
    latency: u32,
}

/// Represents overall bandwidth usage statistics.
#[derive(Serialize)]
struct BandwidthUsage {
    /// Total bandwidth usage in MB/s
    total_bandwidth: u32,
    /// Percentage change in total bandwidth
    total_bandwidth_change: f32,
    /// Incoming bandwidth usage in MB/s
    incoming_bandwidth: u32,
    /// Percentage change in incoming bandwidth
    incoming_bandwidth_change: f32,
    /// Outgoing bandwidth usage in MB/s
    outgoing_bandwidth: u32,
    /// Percentage change in outgoing bandwidth
    outgoing_bandwidth_change: f32,
}

/// Represents bandwidth usage for a specific cluster.
#[derive(Serialize)]
struct ClusterBandwidth {
    /// Name of the cluster
    name: String,
    /// Bandwidth usage in MB/s
    bandwidth: u32,
    /// Percentage change in bandwidth usage
    change: f32,
}

/// Represents bandwidth usage for a specific server.
#[derive(Serialize)]
struct ServerBandwidth {
    /// Name of the server
    name: String,
    /// Name of the cluster this server belongs to
    cluster: String,
    /// Bandwidth usage in MB/s
    bandwidth: u32,
}

/// Represents the health status of a server connection.
#[derive(Serialize)]
struct ConnectionHealth {
    /// Name of the server
    server: String,
    /// Whether the connection is healthy
    healthy: bool,
    /// Ping time in milliseconds
    ping: u32,
    /// Timestamp of the last health check
    last_checked: String,
}

/// Represents information about a system update.
#[derive(Serialize)]
struct UpdateInfo {
    /// Unique identifier for the update
    id: u32,
    /// Name of the update
    name: String,
    /// Version number of the update
    version: String,
    /// Size of the update package
    size: String,
    /// Importance level of the update
    importance: String,
}

/// Represents a historical record of a system update.
#[derive(Serialize)]
struct UpdateHistory {
    /// Unique identifier for the update record
    id: u32,
    /// Name of the update
    name: String,
    /// Version number of the update
    version: String,
    /// Date when the update was applied
    date: String,
    /// Status of the update (e.g., "Successful", "Failed")
    status: String,
}

/// Represents a scheduled maintenance task.
#[derive(Serialize)]
struct ScheduledTask {
    /// Unique identifier for the task
    id: u32,
    /// Name of the task
    name: String,
    /// Detailed description of the task
    description: String,
    /// Cron-style schedule for the task
    schedule: String,
    /// Target system or component for the task
    target: String,
    /// Current status of the task
    status: String,
}

/// Represents a historical record of a maintenance task execution.
#[derive(Serialize)]
struct TaskHistory {
    /// Unique identifier for the task execution record
    id: u32,
    /// Name of the task
    name: String,
    /// Timestamp when the task was executed
    execution_time: String,
    /// Status of the task execution
    status: String,
    /// Duration of the task execution
    duration: String,
}

/// Represents information about a system backup.
#[derive(Serialize)]
struct Backup {
    /// Unique identifier for the backup
    id: u32,
    /// Name of the backup
    name: String,
    /// Date when the backup was created
    date: String,
    /// Size of the backup
    size: String,
    /// Status of the backup
    status: String,
}

/// Represents the load balancing policy configuration.
#[derive(Serialize)]
struct LoadBalancingPolicy {
    /// Maximum number of players per region
    region_size: u32,
    /// Player threshold for creating a new shard
    shard_threshold: u32,
    /// Maximum number of players per server
    max_players_per_server: u32,
    /// Player threshold for spawning a new server
    server_spawn_threshold: u32,
}

/// Represents user access information and permissions.
#[derive(Serialize)]
struct UserAccess {
    /// Unique identifier for the user
    id: u32,
    /// Name of the user
    name: String,
    /// Email address of the user
    email: String,
    /// Role of the user in the system
    role: String,
    /// List of permissions granted to the user
    permissions: Vec<String>,
}

/// Represents an entry in the system audit log.
#[derive(Serialize)]
struct AuditLog {
    /// Unique identifier for the log entry
    id: u32,
    /// Timestamp of the logged action
    timestamp: DateTime<Utc>,
    /// User who performed the action
    user: String,
    /// Type of action performed
    action: String,
    /// Resource affected by the action
    resource: String,
    /// Detailed description of the action
    details: String,
}

/// Represents a subsystem in the application.
#[derive(Serialize)]
struct Subsystem {
    /// Name of the subsystem
    name: String,
    /// Whether the subsystem is currently enabled
    enabled: bool,
    /// Icon representing the subsystem
    icon: String,
    /// Configuration options for the subsystem
    config: Vec<SubsystemConfig>,
}

/// Represents a configuration option for a subsystem.
#[derive(Serialize)]
struct SubsystemConfig {
    /// Name of the configuration option
    name: String,
    /// Type of the configuration option (e.g., "text", "number")
    config_type: String,
    /// Possible values for the configuration option (if applicable)
    options: Option<Vec<String>>,
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

/// Handles GET requests for cluster usage data.
///
/// This endpoint provides CPU and memory usage data for the cluster over time.
#[get("/cluster-usage")]
async fn cluster_usage() -> impl Responder {
    let usage = ClusterUsage {
        labels: vec!["00:00", "02:00", "04:00", "06:00", "08:00", "10:00", "12:00", "14:00", "16:00", "18:00", "20:00", "22:00"]
            .into_iter().map(String::from).collect(),
        cpu_usage: generate_random_data(50.0, 90.0, 12),
        memory_usage: generate_random_data(40.0, 80.0, 12),
    };
    web::Json(usage)
}

/// Handles GET requests for server information.
///
/// This endpoint retrieves server data from the database and returns it as JSON.
#[get("/servers")]
async fn get_servers(pool: web::Data<SqlitePool>) -> impl Responder {
    let query = "SELECT id, name, status, players, cpu, memory FROM servers";
    let rows = sqlx::query(query)
        .fetch_all(pool.get_ref())
        .await
        .unwrap_or_else(|_| vec![]); // Return an empty vector in case of error

    let servers: Vec<Server> = rows.into_iter().map(|row| Server {
        name: row.get("name"),
        status: row.get("status"),
        players: row.get("players"),
        cpu: row.get("cpu"),
        memory: row.get("memory"),
    }).collect();

    web::Json(servers)
}

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

/// Handles GET requests for deployment information.
///
/// This endpoint provides details about current game deployments.
#[get("/deployments")]
async fn deployments() -> impl Responder {
    let deployments = vec![
        DeploymentInfo {
            name: "Test Deploy".to_string(),
            region: "US-East".to_string(),
            avg_load: 2.7931,
            avg_latency: "20ms".to_string(),
            inbound_traffic: "47.3 mb/s".to_string(),
            outbound_traffic: "56.2 mb/s".to_string(),
            players: 555,
            status: "Online".to_string(),
        },
        // Add more deployments as needed
    ];
    web::Json(deployments)
}

/// Handles GET requests for database information.
///
/// This endpoint provides details about the database instances used by the system.
#[get("/databases")]
async fn databases() -> impl Responder {
    let databases = vec![
        DatabaseInfo {
            name: "Test Deploy Database".to_string(),
            region: "US-East".to_string(),
            size: "2.7 GB".to_string(),
            db_type: "Graph".to_string(),
            address: "test-deploy-database.myproject.creator.example.com".to_string(),
            queries_per_second: 41,
            active_connections: 537,
        },
        // Add more database entries as needed
    ];
    web::Json(databases)
}

/// Handles GET requests for system alerts.
///
/// This endpoint provides a list of current system alerts or notifications.
#[get("/alerts")]
async fn alerts() -> impl Responder {
    let alerts = vec![
        AlertInfo {
            id: 1,
            clusters: vec!["Cluster A".to_string(), "Cluster B".to_string()],
            servers: vec!["Server 1".to_string(), "Server 3".to_string()],
            level: "ERROR".to_string(),
            description: "CPU usage high".to_string(),
            timestamp: Utc::now(),
        },
        // Add more alerts as needed
    ];
    web::Json(alerts)
}

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

/// Handles GET requests for network region information.
///
/// This endpoint provides a list of network regions with their geographic and latency data.
#[get("/network/regions")]
async fn network_regions() -> impl Responder {
    let regions = vec![
        Region { name: "North America".to_string(), lat: 40.0, lon: -100.0, latency: 30 },
        Region { name: "Europe".to_string(), lat: 50.0, lon: 10.0, latency: 45 },
        Region { name: "Asia".to_string(), lat: 30.0, lon: 100.0, latency: 60 },
        Region { name: "South America".to_string(), lat: -20.0, lon: -60.0, latency: 55 },
        Region { name: "Australia".to_string(), lat: -25.0, lon: 135.0, latency: 70 },
    ];
    web::Json(regions)
}

/// Handles GET requests for network bandwidth usage.
///
/// This endpoint provides overall bandwidth usage statistics.
#[get("/network/bandwidth")]
async fn network_bandwidth() -> impl Responder {
    let bandwidth = BandwidthUsage {
        total_bandwidth: 400,
        total_bandwidth_change: 14.3,
        incoming_bandwidth: 180,
        incoming_bandwidth_change: 5.7,
        outgoing_bandwidth: 220,
        outgoing_bandwidth_change: 8.6,
    };
    web::Json(bandwidth)
}

/// Handles GET requests for cluster-specific bandwidth information.
///
/// This endpoint provides bandwidth usage data for individual clusters.
#[get("/network/cluster-bandwidth")]
async fn cluster_bandwidth() -> impl Responder {
    let clusters: Vec<ClusterBandwidth> = (0..10).map(|i| ClusterBandwidth {
        name: format!("Cluster {}", (65 + i) as u8 as char),
        bandwidth: rand::thread_rng().gen_range(50..250),
        change: rand::thread_rng().gen_range(-5.0..5.0),
    }).collect();
    web::Json(clusters)
}

/// Handles GET requests for server-specific bandwidth information.
///
/// This endpoint provides bandwidth usage data for individual servers.
#[get("/network/server-bandwidth")]
async fn server_bandwidth() -> impl Responder {
    let servers: Vec<ServerBandwidth> = (0..20).map(|i| ServerBandwidth {
        name: format!("Server {}", i + 1),
        cluster: format!("Cluster {}", (65 + (i % 5)) as u8 as char),
        bandwidth: rand::thread_rng().gen_range(10..60),
    }).collect();
    web::Json(servers)
}

/// Handles GET requests for network connection health.
///
/// This endpoint provides health status information for server connections.
#[get("/network/health")]
async fn connection_health() -> impl Responder {
    let health: Vec<ConnectionHealth> = (1..=10).map(|i| ConnectionHealth {
        server: format!("Server {}", i),
        healthy: rand::thread_rng().gen_bool(0.8),
        ping: rand::thread_rng().gen_range(10..110),
        last_checked: Utc::now().to_rfc3339(),
    }).collect();
    web::Json(health)
}

/// Handles GET requests for system update information.
///
/// This endpoint provides details about available system updates.
#[get("/maintenance/updates")]
async fn system_updates() -> impl Responder {
    let updates = vec![
        UpdateInfo { id: 1, name: "Security Patch".to_string(), version: "1.2.5".to_string(), size: "25 MB".to_string(), importance: "Critical".to_string() },
        UpdateInfo { id: 2, name: "Feature Update".to_string(), version: "2.0.0".to_string(), size: "150 MB".to_string(), importance: "Recommended".to_string() },
        UpdateInfo { id: 3, name: "Bug Fixes".to_string(), version: "1.9.3".to_string(), size: "10 MB".to_string(), importance: "Optional".to_string() },
    ];
    web::Json(updates)
}

/// Handles GET requests for update history.
///
/// This endpoint provides a history of past system updates.
#[get("/maintenance/update-history")]
async fn update_history() -> impl Responder {
    let history = vec![
        UpdateHistory { id: 1, name: "Hotfix".to_string(), version: "1.2.4".to_string(), date: "2024-08-01".to_string(), status: "Successful".to_string() },
        UpdateHistory { id: 2, name: "Performance Update".to_string(), version: "1.9.2".to_string(), date: "2024-07-15".to_string(), status: "Successful".to_string() },
        UpdateHistory { id: 3, name: "Security Update".to_string(), version: "1.2.3".to_string(), date: "2024-07-01".to_string(), status: "Failed".to_string() },
    ];
    web::Json(history)
}

/// Handles GET requests for scheduled maintenance tasks.
///
/// This endpoint provides information about scheduled system maintenance tasks.
#[get("/maintenance/tasks")]
async fn scheduled_tasks() -> impl Responder {
    let tasks = vec![
        ScheduledTask { id: 1, name: "Daily Backup".to_string(), description: "Perform daily backup of all databases".to_string(), schedule: "0 0 * * *".to_string(), target: "all".to_string(), status: "Scheduled".to_string() },
        ScheduledTask { id: 2, name: "Log Rotation".to_string(), description: "Rotate and compress log files".to_string(), schedule: "0 1 * * 0".to_string(), target: "nodes".to_string(), status: "Scheduled".to_string() },
        ScheduledTask { id: 3, name: "Security Scan".to_string(), description: "Run weekly security scan".to_string(), schedule: "0 2 * * 1".to_string(), target: "all".to_string(), status: "Scheduled".to_string() },
    ];
    web::Json(tasks)
}

/// Handles GET requests for task execution history.
///
/// This endpoint provides a history of executed maintenance tasks.
#[get("/maintenance/task-history")]
async fn task_history() -> impl Responder {
    let history = vec![
        TaskHistory { id: 1, name: "Daily Backup".to_string(), execution_time: "2024-08-09 00:00".to_string(), status: "Completed".to_string(), duration: "15m 30s".to_string() },
        TaskHistory { id: 2, name: "Log Rotation".to_string(), execution_time: "2024-08-08 01:00".to_string(), status: "Completed".to_string(), duration: "5m 45s".to_string() },
        TaskHistory { id: 3, name: "Security Scan".to_string(), execution_time: "2024-08-07 02:00".to_string(), status: "Failed".to_string(), duration: "30m 0s".to_string() },
    ];
    web::Json(history)
}

/// Handles GET requests for system backup information.
///
/// This endpoint provides details about system backups.
#[get("/maintenance/backups")]
async fn backups() -> impl Responder {
    let backups = vec![
        Backup { id: 1, name: "Full Backup".to_string(), date: "2024-08-09 14:30".to_string(), size: "2.3 GB".to_string(), status: "Completed".to_string() },
        Backup { id: 2, name: "Incremental Backup".to_string(), date: "2024-08-08 22:00".to_string(), size: "500 MB".to_string(), status: "Completed".to_string() },
        Backup { id: 3, name: "Database Backup".to_string(), date: "2024-08-07 03:00".to_string(), size: "1.1 GB".to_string(), status: "Completed".to_string() },
    ];
    web::Json(backups)
}

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

/// Handles GET requests for the system audit log.
///
/// This endpoint provides entries from the system audit log.
#[get("/security/audit-log")]
async fn audit_log() -> impl Responder {
    let logs = vec![
        AuditLog { id: 1, timestamp: Utc::now(), user: "Alice".to_string(), action: "Create".to_string(), resource: "New Server Instance".to_string(), details: "Created server instance 'US-West-01'".to_string() },
        AuditLog { id: 2, timestamp: Utc::now(), user: "Bob".to_string(), action: "Edit".to_string(), resource: "Load Balancing Policy".to_string(), details: "Updated region size from 1000m to 1500m".to_string() },
        AuditLog { id: 3, timestamp: Utc::now(), user: "Charlie".to_string(), action: "Remove".to_string(), resource: "User Account".to_string(), details: "Removed user 'inactive_user_123'".to_string() },
    ];
    web::Json(logs)
}

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
            .service(cluster_usage)
            .service(get_servers)
            .service(player_activities)
            .service(deployments)
            .service(databases)
            .service(alerts)
            .service(network_latency)
            .service(network_regions)
            .service(network_bandwidth)
            .service(cluster_bandwidth)
            .service(server_bandwidth)
            .service(connection_health)
            .service(system_updates)
            .service(update_history)
            .service(scheduled_tasks)
            .service(task_history)
            .service(backups)
            .service(load_balancing_policy)
            .service(user_access)
            .service(audit_log)
            .service(subsystems)
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