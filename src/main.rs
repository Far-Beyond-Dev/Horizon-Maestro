use std::path::{Path, PathBuf};
use std::fs;
use serde_derive::{Deserialize, Serialize};
use tokio::process::Command;
use tokio::sync::oneshot;
use colored::*;
use std::error::Error;
use std::fmt;
use futures::future::join_all;

mod api;
mod handlers;

/// Main configuration structure for Maestro
#[derive(Deserialize, Serialize, Clone)]
struct Config {
    npm: NpmConfig,
    docker: DockerConfig,
    deployment: DeploymentConfig,
    cache: Option<String>,
}

/// Custom error type for Maestro operations
#[derive(Debug)]
struct MaestroError(String);

impl fmt::Display for MaestroError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}

impl Error for MaestroError {}

/// Configuration for NPM-related operations
#[derive(Deserialize, Serialize, Clone)]
struct NpmConfig {
    dashboard_path: String,
}

/// Configuration for deployment hosts
#[derive(Deserialize, Serialize, Clone)]
struct DeploymentConfig {
    hosts: Vec<Host>,
    parallel_containers: bool,
}

/// Represents a host for deployment
#[derive(Deserialize, Serialize, Clone)]
struct Host {
    address: String,
    username: String,
    auth_method: AuthMethod,
    ssh_port: Option<u16>,
}

/// Authentication methods for SSH connections
#[derive(Deserialize, Serialize, Clone)]
enum AuthMethod {
    Password(String),
    Key(String),
}

/// Configuration for Docker containers
#[derive(Deserialize, Serialize, Clone)]
struct DockerConfig {
    containers: Vec<ContainerConfig>,
    instances: u32,  // New field to control the number of instances
}


/// Configuration for individual Docker containers
#[derive(Deserialize, Serialize, Clone)]
struct ContainerConfig {
    image_name: String,
    container_name: String,
}

/// Port number for the dashboard preview server
const DASHBOARD_PORT: u16 = 3000;

/// File name used to indicate that the dashboard has been built
const BUILT_INDICATOR: &str = ".built";

/// Runs an npm command in the specified directory.
async fn run_npm_command(path: &Path, args: &[&str]) -> Result<(), MaestroError> {
    let mut command = Command::new("npm");
    command.args(args)
        .current_dir(path);

    let output = command.output().await
        .map_err(|e| MaestroError(format!("Failed to execute npm command: {}", e)))?;

    if output.status.success() {
        Ok(())
    } else {
        let error = String::from_utf8_lossy(&output.stderr);
        Err(MaestroError(format!("npm command failed: {}", error)))
    }
}

/// Checks if npm is available on the system.
async fn is_npm_available() -> bool {
    Command::new("npm")
        .arg("--version")
        .output()
        .await
        .is_ok()
}

/// Deploys containers locally using Docker.
async fn deploy_locally(config: &Config) -> Result<(), MaestroError> {
    println!("{}", "🏠 Deploying locally".blue().bold());

    let container_tasks: Vec<_> = config.docker.containers
        .iter()
        .map(|container| {
            let container = container.clone();
            tokio::spawn(async move {
                deploy_container_locally(&container).await
            })
        })
        .collect();

    let results = join_all(container_tasks).await;

    for result in results {
        match result {
            Ok(Ok(())) => {},
            Ok(Err(e)) => return Err(e),
            Err(e) => return Err(MaestroError(format!("Task panicked: {}", e))),
        }
    }

    println!("{}", "✅ Deployed locally".green().bold());
    Ok(())
}

/// Deploys a single container locally
async fn deploy_container_locally(container: &ContainerConfig) -> Result<(), MaestroError> {
    let docker_pull = Command::new("docker")
        .args(&["pull", &container.image_name])
        .output()
        .await
        .map_err(|e| MaestroError(format!("Failed to pull Docker image {}: {}", container.image_name, e)))?;

    if !docker_pull.status.success() {
        let error = String::from_utf8_lossy(&docker_pull.stderr);
        return Err(MaestroError(format!("Failed to pull Docker image {}: {}", container.image_name, error)));
    }

    let _ = Command::new("docker")
        .args(&["rm", "-f", &container.container_name])
        .output()
        .await;

    let docker_run = Command::new("docker")
        .args(&[
            "run",
            "-d",
            "--name", &container.container_name,
            &container.image_name
        ])
        .output()
        .await
        .map_err(|e| MaestroError(format!("Failed to run Docker container {}: {}", container.container_name, e)))?;

    if !docker_run.status.success() {
        let error = String::from_utf8_lossy(&docker_run.stderr);
        return Err(MaestroError(format!("Failed to run Docker container {}: {}", container.container_name, error)));
    }

    let docker_ps = Command::new("docker")
        .args(&["ps", "--filter", &format!("name={}", container.container_name), "--format", "{{.Names}}"])
        .output()
        .await
        .map_err(|e| MaestroError(format!("Failed to verify container {}: {}", container.container_name, e)))?;

    let container_name = String::from_utf8_lossy(&docker_ps.stdout).trim().to_string();
    if container_name == container.container_name {
        println!("{}", format!("✅ Container '{}' is running", container.container_name).green().bold());
        Ok(())
    } else {
        Err(MaestroError(format!("Container '{}' is not running", container.container_name)))
    }
}

/// Deploys containers to a remote host.
async fn deploy_remotely(host: &Host, config: &Config) -> Result<(), MaestroError> {
    println!("{}", format!("🌐 Deploying to {}", host.address).blue().bold());

    if config.deployment.parallel_containers {
        let container_tasks: Vec<_> = config.docker.containers
            .iter()
            .flat_map(|container| {
                (0..config.docker.instances).map(move |instance| {
                    let container = container.clone();
                    let host = host.clone();
                    let instance = instance;
                    tokio::spawn(async move {
                        deploy_container_remotely(&host, &container, instance).await
                    })
                })
            })
            .collect();

        let results = join_all(container_tasks).await;

        for result in results {
            match result {
                Ok(Ok(())) => {},
                Ok(Err(e)) => return Err(e),
                Err(e) => return Err(MaestroError(format!("Task panicked: {}", e))),
            }
        }
    } else {
        for container in &config.docker.containers {
            for instance in 0..config.docker.instances {
                deploy_container_remotely(host, container, instance).await?;
            }
        }
    }

    println!("{}", format!("✅ All containers deployed to {}", host.address).green().bold());
    Ok(())
}

/// Deploys a single container instance to a remote host
async fn deploy_container_remotely(host: &Host, container: &ContainerConfig, instance: u32) -> Result<(), MaestroError> {
    let instance_name = format!("{}-{}", container.container_name, instance);
    
    let docker_commands = vec![
        format!("docker pull {}", container.image_name),
        format!("docker rm -f {}", instance_name),
        format!(
            "docker run -d --name {} {}",
            instance_name, container.image_name
        ),
        format!(
            "docker ps --filter name={} --format '{{{{.Names}}}}'",
            instance_name
        ),
    ];

    for cmd in docker_commands {
        let output = run_ssh_command(&cmd, host).await?;
        println!("SSH OUTPUT | {}@{}:{} / $ {}", host.username, host.address, host.ssh_port.unwrap_or(22), cmd);
        for line in output.lines() {
            println!("SSH OUTPUT | {}", line);
        }
    }

    println!("{}", format!("✅ Container '{}' (instance {}) deployed to {}", container.container_name, instance, host.address).green().bold());
    Ok(())
}

/// Ensures Docker is installed on a remote host.
async fn ensure_docker_installed_remote(host: &Host) -> Result<(), MaestroError> {
    println!("{}", format!("🐳 Checking Docker installation on {}...", host.address).blue().bold());

    let check_docker = run_ssh_command("command -v docker || echo 'Docker not found'", host).await?;

    if check_docker.contains("Docker not found") {
        println!("{}", format!("⚠️  Docker is not installed on {}. Attempting to install...", host.address).yellow().bold());
        install_docker_remote(host).await
    } else {
        println!("{}", format!("✅ Docker is installed on {}", host.address).green().bold());
        Ok(())
    }
}

/// Ensures Docker is installed locally.
async fn ensure_docker_installed_local() -> Result<(), MaestroError> {
    println!("{}", "🐳 Checking Docker installation locally...".blue().bold());

    let docker_check = Command::new("docker")
        .arg("--version")
        .output()
        .await;

    match docker_check {
        Ok(output) if output.status.success() => {
            let version = String::from_utf8_lossy(&output.stdout);
            println!("{}", format!("✅ Docker is installed locally: {}", version.trim()).green().bold());
            Ok(())
        },
        _ => {
            println!("{}", "⚠️  Docker is not installed locally. Attempting to install...".yellow().bold());
            install_docker_local().await
        }
    }
}

/// Installs Docker locally.
async fn install_docker_local() -> Result<(), MaestroError> {
    println!("{}", "📥 Downloading Docker installation script...".blue().bold());
    
    let curl_output = Command::new("curl")
        .arg("-fsSL")
        .arg("https://get.docker.com")
        .output()
        .await
        .map_err(|e| MaestroError(format!("Failed to download Docker installation script: {}", e)))?;

    if !curl_output.status.success() {
        return Err(MaestroError("Failed to download Docker installation script".to_string()));
    }

    let script_content = String::from_utf8_lossy(&curl_output.stdout).to_string();

    println!("{}", "🚀 Running Docker installation script locally...".blue().bold());

    let install_output = Command::new("sh")
        .arg("-c")
        .arg(&script_content)
        .output()
        .await
        .map_err(|e| MaestroError(format!("Failed to run Docker installation script: {}", e)))?;

    if install_output.status.success() {
        println!("{}", "✅ Docker installed successfully on local machine".green().bold());
        Ok(())
    } else {
        let error = String::from_utf8_lossy(&install_output.stderr);
        Err(MaestroError(format!("Docker installation failed locally: {}", error)))
    }
}
    
/// Installs Docker on a remote host.
async fn install_docker_remote(host: &Host) -> Result<(), MaestroError> {
    println!("{}", format!("📥 Installing Docker on {}...", host.address).blue().bold());

    let install_command = r#"
        curl -fsSL https://get.docker.com -o get-docker.sh && 
        sudo sh get-docker.sh && 
        sudo usermod -aG docker $USER && 
        echo 'Docker installed successfully'
    "#;
    
    let output = run_ssh_command(install_command, host).await?;

    if output.contains("Docker installed successfully") {
        println!("{}", format!("✅ Docker installed successfully on {}", host.address).green().bold());
        Ok(())
    } else {
        Err(MaestroError(format!("Docker installation failed on {}: {}", host.address, output)))
    }
}

/// Builds the SSH command string based on the host configuration.
fn build_ssh_command(host: &Host) -> String {
    let port_option = host.ssh_port.map_or(String::new(), |port| format!("-p {}", port));
    
    match &host.auth_method {
        AuthMethod::Password(password) => format!(
            "sshpass -p {} ssh {} {}@{} -o StrictHostKeyChecking=no",
            password,
            port_option, host.username, host.address
        ),
        AuthMethod::Key(key_path) => format!(
            "ssh -i {} {} {}@{}",
            key_path, port_option, host.username, host.address
        ),
    }
}

/// Runs an SSH command using the updated build_ssh_command function.
async fn run_ssh_command(command: &str, host: &Host) -> Result<String, MaestroError> {
    let ssh_command = build_ssh_command(host);
    println!("{}", ssh_command);
    let full_command = format!("{} '{}'", ssh_command, command.replace("'", "'\"'\"'")); // Escape command
    println!("{}", full_command);

    let output = Command::new("sh")
        .arg("-c")
        .arg(&full_command)
        .output()
        .await
        .map_err(|e| MaestroError(format!("Failed to execute SSH command: {}", e)))?;

    let stdout = String::from_utf8_lossy(&output.stdout).to_string();
    let stderr = String::from_utf8_lossy(&output.stderr).to_string();

    if output.status.success() {
        Ok(stdout)
    } else {
        Err(MaestroError(format!("Command failed: {}\nStderr: {}", stdout, stderr)))
    }
}

/// Deploys to a specific host (local or remote).
async fn deploy_to_host(host: Host, config: Config) -> Result<(), MaestroError> {
    println!("{}", format!("🚀 Starting deployment to {}", host.address).blue().bold());
    match host.address.as_str() {
        "localhost" => {
            ensure_docker_installed_local().await?;
            deploy_locally(&config).await
        },
        _ => {
            ensure_docker_installed_remote(&host).await?;
            deploy_remotely(&host, &config).await
        }
    }
}

/// Main function to run the Horizon Maestro application.
#[tokio::main]
async fn main() -> Result<(), MaestroError> {
    println!("{}", "🎭 Starting Horizon Maestro".magenta().bold());

    // Read and parse the configuration file
    let config_str = fs::read_to_string("config.toml")
        .map_err(|e| MaestroError(format!("Failed to read config file: {}", e)))?;
    let config: Config = toml::from_str(&config_str)
        .map_err(|e| MaestroError(format!("Failed to parse config file: {}", e)))?;

    // Check if npm is available
    if is_npm_available().await {
        println!("{}", "🎉 npm found!".green().bold());
    } else {
        return Err(MaestroError("❌ npm is not available. Please install Node.js and npm.".to_string()));
    }

    // Create a channel for shutting down the API server
    let (_shutdown_tx, shutdown_rx) = oneshot::channel();
       
    // Start the API server in a new thread
    let _api_handle = tokio::spawn(async move {
        if let Err(e) = api::main::run_api_server(shutdown_rx).await {
            eprintln!("API server error: {}", e);
        }
    });

    // Build or start the dashboard
    build_or_start_dashboard(&config).await?;

    // Deploy to all configured hosts in parallel
    let deployment_tasks: Vec<_> = config.deployment.hosts.clone()
        .into_iter()
        .map(|host| {
            let config_clone = config.clone();
            tokio::spawn(async move {
                let result = deploy_to_host(host.clone(), config_clone).await;
                (host, result)
            })
        })
        .collect();

    let deployment_results = join_all(deployment_tasks).await;

    // Process deployment results
    let mut successful_deployments = 0;
    let total_deployments = deployment_results.len();

    for result in deployment_results {
        match result {
            Ok((host, Ok(()))) => {
                println!("{}", format!("✅ Deployment to {} successful", host.address).green().bold());
                successful_deployments += 1;
            },
            Ok((host, Err(e))) => {
                eprintln!("{}", format!("❌ Deployment to {} failed: {}", host.address, e).red().bold());
            },
            Err(e) => {
                eprintln!("{}", format!("❌ Task panicked: {}", e).red().bold());
            },
        }
    }

    // Print deployment summary
    print_deployment_summary(&config, successful_deployments, total_deployments);
    
    // Keep the application running until interrupted
    println!("\n{}", "🔄 Application is running. Press Ctrl+C to stop.".yellow().bold());
    tokio::signal::ctrl_c().await.map_err(|e| MaestroError(format!("Failed to listen for Ctrl+C: {}", e)))?;
    println!("{}", "👋 Shutting down".magenta().bold());

    Ok(())
}

/// Prints a summary of the deployment.
fn print_deployment_summary(config: &Config, successful_deployments: usize, total_deployments: usize) {
    println!("{}", "📊 Deployment Summary:".cyan().bold());
    println!("   Dashboard: http://localhost:{}", DASHBOARD_PORT);
    println!("   API: http://localhost:{}", 8080);
    println!("   Deployed Containers:");
    for container in &config.docker.containers {
        println!("     - Image: {}", container.image_name);
        println!("       Container Name: {} (x{} instances)", container.container_name, config.docker.instances);
    }
    println!("   Deployment Results:");
    println!("     - Successful: {}/{}", successful_deployments, total_deployments);
    println!("     - Failed: {}/{}", total_deployments - successful_deployments, total_deployments);
    
    println!("\n{}", "🔍 Notes:".yellow().bold());
    println!("   - Some containers may exit immediately after running if they're designed for short-lived tasks.");
    println!("   - Use 'docker ps -a' to see all containers, including stopped ones.");
    println!("   - To view container logs, use: docker logs <container_name-instance>");
}