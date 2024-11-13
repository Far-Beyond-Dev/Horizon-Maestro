use crate::{Config, Host, MaestroError};
use tokio::process::Command;
use futures::future::join_all;
use colored::*;
use std::fs;

/// Reads and parses the configuration file.
///
/// # Arguments
/// * `path` - A string slice that holds the path to the configuration file
///
/// # Returns
/// - `Ok(Config)` if the file was successfully read and parsed
/// - `Err(MaestroError)` if there was an error reading or parsing the file
pub fn read_config(path: &str) -> Result<Config, MaestroError> {
    let config_str = fs::read_to_string(path)
        .map_err(|e| MaestroError(format!("Failed to read config file: {}", e)))?;
    toml::from_str(&config_str)
        .map_err(|e| MaestroError(format!("Failed to parse config file: {}", e)))
}

/// Deploys to all hosts specified in the configuration.
///
/// # Arguments
/// * `config` - A reference to the Config struct containing deployment details
///
/// # Returns
/// - A vector of Results containing the Host and deployment outcome for each host
pub async fn deploy_to_all_hosts(config: &Config) -> Vec<Result<(Host, Result<(), MaestroError>), tokio::task::JoinError>> {
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

    join_all(deployment_tasks).await
}

/// Deploys to a single host (either local or remote).
///
/// # Arguments
/// * `host` - The Host struct containing connection details
/// * `config` - The Config struct containing deployment details
///
/// # Returns
/// - `Ok(())` if deployment was successful
/// - `Err(MaestroError)` if there was an error during deployment
async fn deploy_to_host(host: Host, config: Config) -> Result<(), MaestroError> {
    println!("{}", format!("🚀 Starting deployment to {}", host.address).blue().bold());
    match host.address.as_str() {
        "localhost" => {
            crate::deployment::docker::ensure_docker_installed_local().await?;
            crate::deployment::docker::deploy_locally(&config).await
        },
        _ => {
            crate::deployment::docker::ensure_docker_installed_remote(&host).await?;
            crate::deployment::docker::deploy_remotely(&host, &config).await
        }
    }
}

/// Processes the results of deployments to all hosts.
///
/// # Arguments
/// * `deployment_results` - A vector of Results containing the Host and deployment outcome for each host
///
/// # Returns
/// - A tuple containing the count of successful deployments and total deployments
pub fn process_deployment_results(
    deployment_results: Vec<Result<(Host, Result<(), MaestroError>), tokio::task::JoinError>>
) -> (usize, usize) {
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

    (successful_deployments, total_deployments)
}

/// Prints a summary of the deployment process.
///
/// # Arguments
/// * `config` - A reference to the Config struct containing deployment details
/// * `successful_deployments` - The number of successful deployments
/// * `total_deployments` - The total number of attempted deployments
pub fn print_deployment_summary(config: &Config, successful_deployments: usize, total_deployments: usize) {
    println!("{}", "📊 Deployment Summary:".cyan().bold());
    println!("   Dashboard: http://localhost:{}", crate::DASHBOARD_PORT);
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

/// Runs an SSH command on a remote host.
///
/// # Arguments
/// * `command` - A string slice containing the command to run
/// * `host` - A reference to the Host struct containing connection details
///
/// # Returns
/// - `Ok(String)` containing the output of the command if successful
/// - `Err(MaestroError)` if there was an error executing the command
pub async fn run_ssh_command(command: &str, host: &Host) -> Result<String, MaestroError> {
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

/// Builds the SSH command string based on the host configuration.
///
/// # Arguments
/// * `host` - A reference to the Host struct containing connection details
///
/// # Returns
/// - A String containing the constructed SSH command
fn build_ssh_command(host: &Host) -> String {
    let port_option = host.ssh_port.map_or(String::new(), |port| format!("-p {}", port));
    
    match &host.auth_method {
        crate::AuthMethod::Password(password) => format!(
            "sshpass -p {} ssh {} {}@{} -o StrictHostKeyChecking=no",
            password,
            port_option, host.username, host.address
        ),
        crate::AuthMethod::Key(key_path) => format!(
            "ssh -i {} {} {}@{}",
            key_path, port_option, host.username, host.address
        ),
    }
}