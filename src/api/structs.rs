use serde::Serialize;
use chrono::{DateTime, Utc};
use std::collections::HashMap;

// pub structs for various data types

/// Represents cluster usage data including CPU and memory usage over time.
#[derive(Serialize)]
pub struct ClusterUsage {
    /// Time labels for the usage data points
    pub labels: Vec<String>,
    /// CPU usage percentages corresponding to each label
    pub cpu_usage: Vec<f64>,
    /// Memory usage percentages corresponding to each label
    pub memory_usage: Vec<f64>,
}

/// Represents information about a game server.
#[derive(Serialize)]
pub struct Server {
    /// Name of the server
    pub name: String,
    /// Current status of the server (e.g., "Online", "Offline")
    pub status: String,
    /// Number of players currently on the server
    pub players: u32,
    /// Current CPU usage percentage
    pub cpu: f64,
    /// Current memory usage percentage
    pub memory: f64,
}

/// Represents a player activity event.
#[derive(Serialize)]
pub struct PlayerActivity {
    /// Name of the player
    pub player: String,
    /// Description of the action performed
    pub action: String,
    /// Time when the action occurred (in human-readable format)
    pub time: String,
    /// URL of the player's avatar image
    pub avatar: String,
}

/// Represents information about a game deployment.
#[derive(Serialize)]
pub struct DeploymentInfo {
    /// Name of the deployment
    pub name: String,
    /// Geographic region of the deployment
    pub region: String,
    /// Average server load
    pub avg_load: f64,
    /// Average latency (in milliseconds)
    pub avg_latency: String,
    /// Inbound network traffic
    pub inbound_traffic: String,
    /// Outbound network traffic
    pub outbound_traffic: String,
    /// Number of players in the deployment
    pub players: u32,
    /// Current status of the deployment
    pub status: String,
}

/// Represents information about a database instance.
#[derive(Serialize)]
pub struct DatabaseInfo {
    /// Name of the database
    pub name: String,
    /// Geographic region of the database
    pub region: String,
    /// Size of the database
    pub size: String,
    /// Type of the database (e.g., "Graph", "Relational")
    pub db_type: String,
    /// Network address of the database
    pub address: String,
    /// Number of queries processed per second
    pub queries_per_second: u32,
    /// Number of active connections to the database
    pub active_connections: u32,
}

/// Represents an alert or notification in the system.
#[derive(Serialize)]
pub struct AlertInfo {
    /// Unique identifier for the alert
    pub id: u32,
    /// List of affected clusters
    pub clusters: Vec<String>,
    /// List of affected servers
    pub servers: Vec<String>,
    /// Severity level of the alert
    pub level: String,
    /// Detailed description of the alert
    pub description: String,
    /// Timestamp when the alert was generated
    pub timestamp: DateTime<Utc>,
}

/// Represents network latency statistics.
#[derive(Serialize)]
pub struct NetworkLatency {
    /// Average latency in milliseconds
    pub avg_latency: u32,
    /// Peak latency in milliseconds
    pub peak_latency: u32,
    /// Packet loss percentage
    pub packet_loss: f32,
    /// Historical latency data over time
    pub latency_over_time: Vec<u32>,
    /// Change in average latency compared to previous period
    pub avg_latency_change: f32,
    /// Change in peak latency compared to previous period
    pub peak_latency_change: f32,
    /// Change in packet loss compared to previous period
    pub packet_loss_change: f32,
    /// Trend of peak latency over time
    pub peak_latency_trend: Vec<u32>,
    /// Distribution of latency across different ranges
    pub latency_distribution: HashMap<String, u32>,
}

/// Represents a geographic region with network statistics.
#[derive(Serialize)]
pub struct Region {
    /// Name of the region
    pub name: String,
    /// Latitude coordinate
    pub lat: f32,
    /// Longitude coordinate
    pub lon: f32,
    /// Average latency to this region in milliseconds
    pub latency: u32,
}

/// Represents overall bandwidth usage statistics.
#[derive(Serialize)]
pub struct BandwidthUsage {
    /// Total bandwidth usage in MB/s
    pub total_bandwidth: u32,
    /// Percentage change in total bandwidth
    pub total_bandwidth_change: f32,
    /// Incoming bandwidth usage in MB/s
    pub incoming_bandwidth: u32,
    /// Percentage change in incoming bandwidth
    pub incoming_bandwidth_change: f32,
    /// Outgoing bandwidth usage in MB/s
    pub outgoing_bandwidth: u32,
    /// Percentage change in outgoing bandwidth
    pub outgoing_bandwidth_change: f32,
}

/// Represents bandwidth usage for a specific cluster.
#[derive(Serialize)]
pub struct ClusterBandwidth {
    /// Name of the cluster
    pub name: String,
    /// Bandwidth usage in MB/s
    pub bandwidth: u32,
    /// Percentage change in bandwidth usage
    pub change: f32,
}

/// Represents bandwidth usage for a specific server.
#[derive(Serialize)]
pub struct ServerBandwidth {
    /// Name of the server
    pub name: String,
    /// Name of the cluster this server belongs to
    pub cluster: String,
    /// Bandwidth usage in MB/s
    pub bandwidth: u32,
}

/// Represents the health status of a server connection.
#[derive(Serialize)]
pub struct ConnectionHealth {
    /// Name of the server
    pub server: String,
    /// Whether the connection is healthy
    pub healthy: bool,
    /// Ping time in milliseconds
    pub ping: u32,
    /// Timestamp of the last health check
    pub last_checked: String,
}

/// Represents information about a system update.
#[derive(Serialize)]
pub struct UpdateInfo {
    /// Unique identifier for the update
    pub id: u32,
    /// Name of the update
    pub name: String,
    /// version number of the update
    pub version: String,
    /// Size of the update package
    pub size: String,
    /// Importance level of the update
    pub importance: String,
}

/// Represents a historical record of a system update.
#[derive(Serialize)]
pub struct UpdateHistory {
    /// Unique identifier for the update record
    pub id: u32,
    /// Name of the update
    pub name: String,
    /// version number of the update
    pub version: String,
    /// Date when the update was applied
    pub date: String,
    /// Status of the update (e.g., "Successful", "Failed")
    pub status: String,
}

/// Represents a scheduled maintenance task.
#[derive(Serialize)]
pub struct ScheduledTask {
    /// Unique identifier for the task
    pub id: u32,
    /// Name of the task
    pub name: String,
    /// Detailed description of the task
    pub description: String,
    /// Cron-style schedule for the task
    pub schedule: String,
    /// Target system or component for the task
    pub target: String,
    /// Current status of the task
    pub status: String,
}

/// Represents a historical record of a maintenance task execution.
#[derive(Serialize)]
pub struct TaskHistory {
    /// Unique identifier for the task execution record
    pub id: u32,
    /// Name of the task
    pub name: String,
    /// Timestamp when the task was executed
    pub execution_time: String,
    /// Status of the task execution
    pub status: String,
    /// Duration of the task execution
    pub duration: String,
}

/// Represents information about a system backup.
#[derive(Serialize)]
pub struct Backup {
    /// Unique identifier for the backup
    pub id: u32,
    /// Name of the backup
    pub name: String,
    /// Date when the backup was created
    pub date: String,
    /// Size of the backup
    pub size: String,
    /// Status of the backup
    pub status: String,
}

/// Represents the load balancing policy configuration.
#[derive(Serialize)]
pub struct LoadBalancingPolicy {
    /// Maximum number of players per region
    pub region_size: u32,
    /// Player threshold for creating a new shard
    pub shard_threshold: u32,
    /// Maximum number of players per server
    pub max_players_per_server: u32,
    /// Player threshold for spawning a new server
    pub server_spawn_threshold: u32,
}

/// Represents user access information and permissions.
#[derive(Serialize)]
pub struct UserAccess {
    /// Unique identifier for the user
    pub id: u32,
    /// Name of the user
    pub name: String,
    /// Email address of the user
    pub email: String,
    /// Role of the user in the system
    pub role: String,
    /// List of permissions granted to the user
    pub permissions: Vec<String>,
}

/// Represents an entry in the system audit log.
#[derive(Serialize)]
pub struct AuditLog {
    /// Unique identifier for the log entry
    pub id: u32,
    /// Timestamp of the logged action
    pub timestamp: DateTime<Utc>,
    /// User who performed the action
    pub user: String,
    /// Type of action performed
    pub action: String,
    /// Resource affected by the action
    pub resource: String,
    /// Detailed description of the action
    pub details: String,
}

/// Represents a subsystem in the application.
#[derive(Serialize)]
pub struct Subsystem {
    /// Name of the subsystem
    pub name: String,
    /// Whether the subsystem is currently enabled
    pub enabled: bool,
    /// Icon representing the subsystem
    pub icon: String,
    /// Configuration options for the subsystem
    pub config: Vec<SubsystemConfig>,
}

/// Represents a configuration option for a subsystem.
#[derive(Serialize)]
pub struct SubsystemConfig {
    /// Name of the configuration option
    pub name: String,
    /// Type of the configuration option (e.g., "text", "number")
    pub config_type: String,
    /// Possible values for the configuration option (if applicable)
    pub options: Option<Vec<String>>,
}