// use std::fs;
// use horizon_data_types::Player;
use serde_derive::{Deserialize, Serialize};
use socketioxide::extract::SocketRef;
use tokio::sync::oneshot;
use colored::*;
use std::error::Error;
use std::fmt;
use std::net::Ipv4Addr;

mod api;
mod system_api;
mod docker_api;
mod deployment;

// Struct to keep track of host data
#[derive(Clone)]
struct GameServer {
    socket: SocketRef,            // A ref to the host's socket, this is used for sending messages or gathering more detailed data about the host
    ip: Ipv4Addr,                 // The IPV4 address of the host, among other things this is used for system repairs over ssh and for health checks
    assigned_region: [i64; 3],    // This value is the RRO (Relative Region Offset) from the center of the world (0,0,0) these are only ever whole numbers
    players_ids: Vec<String>,     // List of all online players connected to this host, commonly used when communicating cross-host
}


#[derive(Deserialize, Serialize, Clone)]
struct Config {
    npm: NpmConfig,
    docker: DockerConfig,
    deployment: DeploymentConfig,
    cache: Option<String>,
}

#[derive(Debug)]
struct MaestroError(String);

impl fmt::Display for MaestroError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}

impl Error for MaestroError {}

#[derive(Deserialize, Serialize, Clone)]
struct NpmConfig {
    dashboard_path: String,
}

#[derive(Deserialize, Serialize, Clone)]
struct DeploymentConfig {
    hosts: Vec<Host>,
    parallel_containers: bool,
}

#[derive(Deserialize, Serialize, Clone)]
struct Host {
    address: String,
    username: String,
    auth_method: AuthMethod,
    ssh_port: Option<u16>,
}

#[derive(Deserialize, Serialize, Clone)]
enum AuthMethod {
    Password(String),
    Key(String),
}

#[derive(Deserialize, Serialize, Clone)]
struct DockerConfig {
    containers: Vec<ContainerConfig>,
    instances: u32,
}

#[derive(Deserialize, Serialize, Clone)]
struct ContainerConfig {
    image_name: String,
    container_name: String,
}

const DASHBOARD_PORT: u16 = 3008;

#[tokio::main]
async fn main() -> Result<(), MaestroError> {
    println!("{}", "🎭 Starting Horizon Maestro".magenta().bold());

    let config: Config = system_api::read_config("config.toml")?;

    let (_shutdown_tx, shutdown_rx) = oneshot::channel();
       
    let _api_handle = tokio::spawn(async move {
        if let Err(e) = api::main::run_api_server(shutdown_rx).await {
            eprintln!("API server error: {}", e);
        }
    });

    let deployment_results = system_api::deploy_to_all_hosts(&config).await;

    let (successful_deployments, total_deployments) = system_api::process_deployment_results(deployment_results);

    system_api::print_deployment_summary(&config, successful_deployments, total_deployments);
    
    println!("\n{}", "🔄 Application is running. Press Ctrl+C to stop.".yellow().bold());
    tokio::signal::ctrl_c().await.map_err(|e| MaestroError(format!("Failed to listen for Ctrl+C: {}", e)))?;
    println!("{}", "👋 Shutting down".magenta().bold());

    Ok(())
}