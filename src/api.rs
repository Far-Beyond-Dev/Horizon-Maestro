use actix_web::{get, web, App, HttpServer, Responder};
use serde::Serialize;
use chrono::{DateTime, Utc};
use rand::Rng;
use std::collections::HashMap;
use tokio::sync::oneshot;

// Structs for various data types

#[derive(Serialize)]
struct ClusterUsage {
    labels: Vec<String>,
    cpu_usage: Vec<f64>,
    memory_usage: Vec<f64>,
}

#[derive(Serialize)]
struct Server {
    name: String,
    status: String,
    players: u32,
    cpu: f64,
    memory: f64,
}

#[derive(Serialize)]
struct PlayerActivity {
    player: String,
    action: String,
    time: String,
    avatar: String,
}

#[derive(Serialize)]
struct DeploymentInfo {
    name: String,
    region: String,
    avg_load: f64,
    avg_latency: String,
    inbound_traffic: String,
    outbound_traffic: String,
    players: u32,
    status: String,
}

#[derive(Serialize)]
struct DatabaseInfo {
    name: String,
    region: String,
    size: String,
    db_type: String,
    address: String,
    queries_per_second: u32,
    active_connections: u32,
}

#[derive(Serialize)]
struct AlertInfo {
    id: u32,
    clusters: Vec<String>,
    servers: Vec<String>,
    level: String,
    description: String,
    timestamp: DateTime<Utc>,
}

#[derive(Serialize)]
struct NetworkLatency {
    avg_latency: u32,
    peak_latency: u32,
    packet_loss: f32,
    latency_over_time: Vec<u32>,
    avg_latency_change: f32,
    peak_latency_change: f32,
    packet_loss_change: f32,
    peak_latency_trend: Vec<u32>,
    latency_distribution: HashMap<String, u32>,
}

#[derive(Serialize)]
struct Region {
    name: String,
    lat: f32,
    lon: f32,
    latency: u32,
}

#[derive(Serialize)]
struct BandwidthUsage {
    total_bandwidth: u32,
    total_bandwidth_change: f32,
    incoming_bandwidth: u32,
    incoming_bandwidth_change: f32,
    outgoing_bandwidth: u32,
    outgoing_bandwidth_change: f32,
}

#[derive(Serialize)]
struct ClusterBandwidth {
    name: String,
    bandwidth: u32,
    change: f32,
}

#[derive(Serialize)]
struct ServerBandwidth {
    name: String,
    cluster: String,
    bandwidth: u32,
}

#[derive(Serialize)]
struct ConnectionHealth {
    server: String,
    healthy: bool,
    ping: u32,
    last_checked: String,
}

#[derive(Serialize)]
struct UpdateInfo {
    id: u32,
    name: String,
    version: String,
    size: String,
    importance: String,
}

#[derive(Serialize)]
struct UpdateHistory {
    id: u32,
    name: String,
    version: String,
    date: String,
    status: String,
}

#[derive(Serialize)]
struct ScheduledTask {
    id: u32,
    name: String,
    description: String,
    schedule: String,
    target: String,
    status: String,
}

#[derive(Serialize)]
struct TaskHistory {
    id: u32,
    name: String,
    execution_time: String,
    status: String,
    duration: String,
}

#[derive(Serialize)]
struct Backup {
    id: u32,
    name: String,
    date: String,
    size: String,
    status: String,
}

#[derive(Serialize)]
struct LoadBalancingPolicy {
    region_size: u32,
    shard_threshold: u32,
    max_players_per_server: u32,
    server_spawn_threshold: u32,
}

#[derive(Serialize)]
struct UserAccess {
    id: u32,
    name: String,
    email: String,
    role: String,
    permissions: Vec<String>,
}

#[derive(Serialize)]
struct AuditLog {
    id: u32,
    timestamp: DateTime<Utc>,
    user: String,
    action: String,
    resource: String,
    details: String,
}

#[derive(Serialize)]
struct Subsystem {
    name: String,
    enabled: bool,
    icon: String,
    config: Vec<SubsystemConfig>,
}

#[derive(Serialize)]
struct SubsystemConfig {
    name: String,
    config_type: String,
    options: Option<Vec<String>>,
}

// Helper function to generate random data
fn generate_random_data<T>(min: T, max: T, count: usize) -> Vec<T>
where
    T: rand::distributions::uniform::SampleUniform + Copy + PartialOrd,
{
    let mut rng = rand::thread_rng();
    (0..count).map(|_| rng.gen_range(min..=max)).collect()
}

// Routes

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

#[get("/servers")]
async fn get_servers() -> impl Responder {
    let servers: Vec<Server> = (1..=5).map(|i| Server {
        name: format!("Server {}", i),
        status: if i % 3 == 0 { "Maintenance".to_string() } else { "Online".to_string() },
        players: rand::thread_rng().gen_range(100..1000),
        cpu: rand::thread_rng().gen_range(20.0..80.0),
        memory: rand::thread_rng().gen_range(30.0..90.0),
    }).collect();
    web::Json(servers)
}

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

#[get("/network/cluster-bandwidth")]
async fn cluster_bandwidth() -> impl Responder {
    let clusters: Vec<ClusterBandwidth> = (0..10).map(|i| ClusterBandwidth {
        name: format!("Cluster {}", (65 + i) as u8 as char),
        bandwidth: rand::thread_rng().gen_range(50..250),
        change: rand::thread_rng().gen_range(-5.0..5.0),
    }).collect();
    web::Json(clusters)
}

#[get("/network/server-bandwidth")]
async fn server_bandwidth() -> impl Responder {
    let servers: Vec<ServerBandwidth> = (0..20).map(|i| ServerBandwidth {
        name: format!("Server {}", i + 1),
        cluster: format!("Cluster {}", (65 + (i % 5)) as u8 as char),
        bandwidth: rand::thread_rng().gen_range(10..60),
    }).collect();
    web::Json(servers)
}

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

#[get("/maintenance/updates")]
async fn system_updates() -> impl Responder {
    let updates = vec![
        UpdateInfo { id: 1, name: "Security Patch".to_string(), version: "1.2.5".to_string(), size: "25 MB".to_string(), importance: "Critical".to_string() },
        UpdateInfo { id: 2, name: "Feature Update".to_string(), version: "2.0.0".to_string(), size: "150 MB".to_string(), importance: "Recommended".to_string() },
        UpdateInfo { id: 3, name: "Bug Fixes".to_string(), version: "1.9.3".to_string(), size: "10 MB".to_string(), importance: "Optional".to_string() },
    ];
    web::Json(updates)
}

#[get("/maintenance/update-history")]
async fn update_history() -> impl Responder {
    let history = vec![
        UpdateHistory { id: 1, name: "Hotfix".to_string(), version: "1.2.4".to_string(), date: "2024-08-01".to_string(), status: "Successful".to_string() },
        UpdateHistory { id: 2, name: "Performance Update".to_string(), version: "1.9.2".to_string(), date: "2024-07-15".to_string(), status: "Successful".to_string() },
        UpdateHistory { id: 3, name: "Security Update".to_string(), version: "1.2.3".to_string(), date: "2024-07-01".to_string(), status: "Failed".to_string() },
    ];
    web::Json(history)
}

#[get("/maintenance/tasks")]
async fn scheduled_tasks() -> impl Responder {
    let tasks = vec![
        ScheduledTask { id: 1, name: "Daily Backup".to_string(), description: "Perform daily backup of all databases".to_string(), schedule: "0 0 * * *".to_string(), target: "all".to_string(), status: "Scheduled".to_string() },
        ScheduledTask { id: 2, name: "Log Rotation".to_string(), description: "Rotate and compress log files".to_string(), schedule: "0 1 * * 0".to_string(), target: "nodes".to_string(), status: "Scheduled".to_string() },
        ScheduledTask { id: 3, name: "Security Scan".to_string(), description: "Run weekly security scan".to_string(), schedule: "0 2 * * 1".to_string(), target: "all".to_string(), status: "Scheduled".to_string() },
    ];
    web::Json(tasks)
}

#[get("/maintenance/task-history")]
async fn task_history() -> impl Responder {
    let history = vec![
        TaskHistory { id: 1, name: "Daily Backup".to_string(), execution_time: "2024-08-09 00:00".to_string(), status: "Completed".to_string(), duration: "15m 30s".to_string() },
        TaskHistory { id: 2, name: "Log Rotation".to_string(), execution_time: "2024-08-08 01:00".to_string(), status: "Completed".to_string(), duration: "5m 45s".to_string() },
        TaskHistory { id: 3, name: "Security Scan".to_string(), execution_time: "2024-08-07 02:00".to_string(), status: "Failed".to_string(), duration: "30m 0s".to_string() },
    ];
    web::Json(history)
}

#[get("/maintenance/backups")]
async fn backups() -> impl Responder {
    let backups = vec![
        Backup { id: 1, name: "Full Backup".to_string(), date: "2024-08-09 14:30".to_string(), size: "2.3 GB".to_string(), status: "Completed".to_string() },
        Backup { id: 2, name: "Incremental Backup".to_string(), date: "2024-08-08 22:00".to_string(), size: "500 MB".to_string(), status: "Completed".to_string() },
        Backup { id: 3, name: "Database Backup".to_string(), date: "2024-08-07 03:00".to_string(), size: "1.1 GB".to_string(), status: "Completed".to_string() },
    ];
    web::Json(backups)
}

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

#[get("/security/access")]
async fn user_access() -> impl Responder {
    let users = vec![
        UserAccess { id: 1, name: "John Doe".to_string(), email: "john@example.com".to_string(), role: "Admin".to_string(), permissions: vec!["read".to_string(), "write".to_string(), "delete".to_string()] },
        UserAccess { id: 2, name: "Jane Smith".to_string(), email: "jane@example.com".to_string(), role: "Editor".to_string(), permissions: vec!["read".to_string(), "write".to_string()] },
        UserAccess { id: 3, name: "Bob Johnson".to_string(), email: "bob@example.com".to_string(), role: "Viewer".to_string(), permissions: vec!["read".to_string()] },
    ];
    web::Json(users)
}

#[get("/security/audit-log")]
async fn audit_log() -> impl Responder {
    let logs = vec![
        AuditLog { id: 1, timestamp: Utc::now(), user: "Alice".to_string(), action: "Create".to_string(), resource: "New Server Instance".to_string(), details: "Created server instance 'US-West-01'".to_string() },
        AuditLog { id: 2, timestamp: Utc::now(), user: "Bob".to_string(), action: "Edit".to_string(), resource: "Load Balancing Policy".to_string(), details: "Updated region size from 1000m to 1500m".to_string() },
        AuditLog { id: 3, timestamp: Utc::now(), user: "Charlie".to_string(), action: "Remove".to_string(), resource: "User Account".to_string(), details: "Removed user 'inactive_user_123'".to_string() },
    ];
    web::Json(logs)
}

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

pub async fn run_api_server(shutdown_rx: oneshot::Receiver<()>) -> std::io::Result<()> {
    let server = HttpServer::new(|| {
        App::new()
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

    tokio::select! {
        _ = server => {},
        _ = shutdown_rx => {
            println!("Shutting down API server");
        }
    }

    Ok(())
}
