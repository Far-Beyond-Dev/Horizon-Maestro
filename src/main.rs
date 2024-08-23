use std::path::{Path, PathBuf};
use std::fs;
use tokio::process::Command;
use tokio::io::AsyncWriteExt;
use tokio::sync::oneshot;
use colored::*;
use crate::config::{Config, Host, AuthMethod};
use crate::error::MaestroError;

/// Port number for the dashboard preview server
const DASHBOARD_PORT: u16 = 3000;

/// File name used to indicate that the dashboard has been built
const BUILT_INDICATOR: &str = ".built";

/// Builds or starts the dashboard based on the provided configuration.
///
/// This function performs the following steps:
/// 1. Checks if the dashboard directory exists
/// 2. Installs npm dependencies
/// 3. Builds the dashboard if it hasn't been built before
/// 4. Starts the dashboard preview server
///
/// # Arguments
///
/// * `config` - A reference to the Config struct containing application configuration
///
/// # Returns
///
/// * `Ok(())` if the dashboard is successfully built or started
/// * `Err(MaestroError)` if any step fails
///
/// # Errors
///
/// This function will return an error if:
/// - The dashboard directory doesn't exist
/// - npm dependencies fail to install
/// - The dashboard fails to build
/// - The built indicator file can't be created
async fn build_or_start_dashboard(config: &Config) -> Result<(), MaestroError> {
    let dashboard_path = PathBuf::from(&config.npm.dashboard_path);

    // Check if dashboard directory exists
    if !dashboard_path.exists() {
        return Err(MaestroError(format!("Dashboard directory not found at {:?}", dashboard_path)));
    }

    // Install npm dependencies
    println!("{}", "📦 Installing npm dependencies...".yellow().bold());
    run_npm_command(&dashboard_path, &["install"]).await?;
    println!("{}", "✅ npm dependencies installed successfully.".green().bold());

    let built_indicator = dashboard_path.join(BUILT_INDICATOR);

    // Build the dashboard if it hasn't been built before
    if !built_indicator.exists() {
        println!("{}", "🏗️  Building dashboard for the first time...".yellow().bold());
        run_npm_command(&dashboard_path, &["run", "build"]).await?;
        fs::File::create(&built_indicator)
            .map_err(|e| MaestroError(format!("Failed to create built indicator file: {}", e)))?;
        println!("{}", "✅ Dashboard built successfully.".green().bold());
    }

    // Start the dashboard preview
    println!("{}", "🚀 Starting dashboard preview...".cyan().bold());
    tokio::spawn(async move {
        if let Err(e) = run_npm_command(&dashboard_path, &["run", "preview", "--", "--port", &DASHBOARD_PORT.to_string()]).await {
            eprintln!("{}", format!("❌ Error starting dashboard preview: {}", e).red().bold());
        }
    });

    println!("{}", format!("🌐 Dashboard is now running at http://localhost:{}", DASHBOARD_PORT).green().bold());
    Ok(())
}

/// Runs an npm command in the specified directory.
///
/// This function executes an npm command with the given arguments in the specified directory.
///
/// # Arguments
///
/// * `path` - A reference to the Path where the npm command should be executed
/// * `args` - A slice of string slices containing the npm command arguments
///
/// # Returns
///
/// * `Ok(())` if the npm command executes successfully
/// * `Err(MaestroError)` if the command fails to execute or returns a non-zero exit status
///
/// # Errors
///
/// This function will return an error if:
/// - The npm command fails to execute
/// - The npm command returns a non-zero exit status
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
///
/// This function attempts to run the `npm --version` command to determine if npm is installed and accessible.
///
/// # Returns
///
/// * `true` if npm is available (the command executes successfully)
/// * `false` if npm is not available (the command fails to execute)
async fn is_npm_available() -> bool {
    Command::new("npm")
        .arg("--version")
        .output()
        .await
        .is_ok()
}

/// Deploys containers locally using Docker.
///
/// This function performs the following steps for each container specified in the configuration:
/// 1. Pulls the Docker image
/// 2. Removes any existing container with the same name
/// 3. Runs a new container
/// 4. Verifies that the container is running
///
/// # Arguments
///
/// * `config` - A reference to the Config struct containing application configuration
///
/// # Returns
///
/// * `Ok(())` if all containers are successfully deployed
/// * `Err(MaestroError)` if any step fails for any container
///
/// # Errors
///
/// This function will return an error if:
/// - Pulling a Docker image fails
/// - Running a Docker container fails
/// - A container fails to start or is not found after attempting to start it
async fn deploy_locally(config: &Config) -> Result<(), MaestroError> {
    println!("{}", "🏠 Deploying locally".blue().bold());

    for container in &config.docker.containers {
        // Pull the Docker image
        let docker_pull = Command::new("docker")
            .args(&["pull", &container.image_name])
            .output()
            .await
            .map_err(|e| MaestroError(format!("Failed to pull Docker image {}: {}", container.image_name, e)))?;

        if !docker_pull.status.success() {
            let error = String::from_utf8_lossy(&docker_pull.stderr);
            return Err(MaestroError(format!("Failed to pull Docker image {}: {}", container.image_name, error)));
        }

        // Remove existing container if it exists
        let _ = Command::new("docker")
            .args(&["rm", "-f", &container.container_name])
            .output()
            .await;

        // Run the Docker container
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

        // Verify the container is running
        let docker_ps = Command::new("docker")
            .args(&["ps", "--filter", &format!("name={}", container.container_name), "--format", "{{.Names}}"])
            .output()
            .await
            .map_err(|e| MaestroError(format!("Failed to verify container {}: {}", container.container_name, e)))?;

        let container_name = String::from_utf8_lossy(&docker_ps.stdout).trim().to_string();
        if container_name == container.container_name {
            println!("{}", format!("✅ Container '{}' is running", container.container_name).green().bold());
        } else {
            return Err(MaestroError(format!("Container '{}' is not running", container.container_name)));
        }
    }

    println!("{}", "✅ Deployed locally".green().bold());
    Ok(())
}

/// Deploys containers to a remote host.
///
/// This function performs the following steps for each container specified in the configuration:
/// 1. Pulls the Docker image on the remote host
/// 2. Removes any existing container with the same name on the remote host
/// 3. Runs a new container on the remote host
/// 4. Verifies that the container is running on the remote host
///
/// # Arguments
///
/// * `host` - A reference to the Host struct containing remote host information
/// * `config` - A reference to the Config struct containing application configuration
///
/// # Returns
///
/// * `Ok(())` if all containers are successfully deployed to the remote host
/// * `Err(MaestroError)` if any step fails for any container
///
/// # Errors
///
/// This function will return an error if:
/// - SSH connection to the remote host fails
/// - Any Docker command fails on the remote host
/// - A container fails to start or is not found after attempting to start it on the remote host
async fn deploy_remotely(host: &Host, config: &Config) -> Result<(), MaestroError> {
    println!("{}", format!("🌐 Deploying to {}", host.address).blue().bold());

    let ssh_command = build_ssh_command(host);

    for container in &config.docker.containers {
        let docker_commands = vec![
            format!("docker pull {}", container.image_name),
            format!("docker rm -f {}", container.container_name),
            format!(
                "docker run -d --name {} {}",
                container.container_name, container.image_name
            ),
            format!(
                "docker ps --filter name={} --format '{{{{.Names}}}}'",
                container.container_name
            ),
        ];

        for cmd in docker_commands {
            let full_command = format!("{} '{}'", ssh_command, cmd);
            let output = run_ssh_command(&full_command, host).await?;

            println!("Command output: {}", output);
        }

        println!("{}", format!("✅ Container '{}' deployed to {}", container.container_name, host.address).green().bold());
    }

    println!("{}", format!("✅ All containers deployed to {}", host.address).green().bold());
    Ok(())
}

/// Ensures Docker is installed on a remote host.
///
/// This function checks if Docker is installed on the remote host and attempts to install it if it's not present.
///
/// # Arguments
///
/// * `host` - A reference to the Host struct containing remote host information
///
/// # Returns
///
/// * `Ok(())` if Docker is already installed or successfully installed
/// * `Err(MaestroError)` if checking for Docker fails or installation fails
///
/// # Errors
///
/// This function will return an error if:
/// - SSH connection to the remote host fails
/// - Docker version check fails
/// - Docker installation fails
async fn ensure_docker_installed_remote(host: &Host) -> Result<(), MaestroError> {
    println!("{}", format!("🐳 Checking Docker installation on {}...", host.address).blue().bold());

    let ssh_command = build_ssh_command(host);
    let full_command = format!("{} 'docker --version'", ssh_command);

    let output = run_ssh_command(&full_command, host).await?;

    if !output.is_empty() {
        println!("{}", format!("✅ Docker is installed on {}: {}", host.address, output.trim()).green().bold());
        Ok(())
    } else {
        println!("{}", format!("⚠️  Docker is not installed on {}. Attempting to install...", host.address).yellow().bold());
        install_docker_remote(host).await
    }
}

/// Ensures Docker is installed locally.
///
/// This function checks if Docker is installed on the local machine and attempts to install it if it's not present.
///
/// # Returns
///
/// * `Ok(())` if Docker is already installed or successfully installed
/// * `Err(MaestroError)` if checking for Docker fails or installation fails
///
/// # Errors
///
/// This function will return an error if:
/// - Docker version check fails
/// - Docker installation fails
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
///
/// This function downloads the Docker installation script and runs it to install Docker on the local machine.
///
/// # Returns
///
/// * `Ok(())` if Docker is successfully installed
/// * `Err(MaestroError)` if the installation fails at any step
///
/// # Errors
///
/// This function will return an error if:
/// - Downloading the Docker installation script fails
/// - Running the Docker installation script fails
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
///
/// This function runs a Docker installation script on the remote host to install Docker.
///
/// # Arguments
///
/// * `host` - A reference to the Host struct containing remote host information
///
/// # Returns
///
/// * `Ok(())` if Docker is successfully installed on the remote host
/// * `Err(MaestroError)` if the installation fails
///
/// # Errors
///
/// This function will return an error if:
/// - SSH connection to the remote host fails
/// - Downloading or running the Docker installation script fails
async fn install_docker_remote(host: &Host) -> Result<(), MaestroError> {
    println!("{}", format!("📥 Installing Docker on {}...", host.address).blue().bold());

    let install_command = r#"curl -fsSL https://get.docker.com -o get-docker.sh && sudo sh get-docker.sh"#;
    
    let ssh_command = build_ssh_command(host);
    let full_command = format!("{} '{}'", ssh_command, install_command);

    let output = run_ssh_command(&full_command, host).await?;

    if !output.contains("ERROR") {
        println!("{}", format!("✅ Docker installed successfully on {}", host.address).green().bold());
        Ok(())
    } else {
        Err(MaestroError(format!("Docker installation failed on {}: {}", host.address, output)))
    }
}

/// Builds the SSH command string based on the host configuration.
///
/// This function constructs an SSH command string with appropriate options based on the host's
/// authentication method and SSH port.
///
/// # Arguments
///
/// * `host` - A reference to the Host struct containing remote host information
///
/// # Returns
///
/// * A String containing the constructed SSH command
fn build_ssh_command(host: &Host) -> String {
    let port_option = host.ssh_port.map_or(String::new(), |port| format!("-p {}", port));
    
    match &host.auth_method {
        AuthMethod::Password(_) => format!(
            "ssh -o BatchMode=no -o StrictHostKeyChecking=no {} {}@{}",
            port_option, host.username, host.address
        ),
        AuthMethod::Key(key_path) => format!(
            "ssh -i {} {} {}@{}",
            key_path, port_option, host.username, host.address
        ),
    }
}

/// Runs an SSH command and handles password input if necessary.
///
/// This function executes an SSH command on a remote host, handling password-based
/// authentication if required.
///
/// # Arguments
///
/// * `command` - A string slice containing the command to be executed on the remote host
/// * `host` - A reference to the Host struct containing remote host information
///
/// # Returns
///
/// * `Ok(String)` containing the command output if successful
/// * `Err(MaestroError)` if the command execution fails
///
/// # Errors
///
/// This function will return an error if:
/// - The SSH command fails to execute
/// - The remote command returns a non-zero exit status
async fn run_ssh_command(command: &str, host: &Host) -> Result<String, MaestroError> {
    let ssh_command = build_ssh_command(host);
    let full_command = format!("{} '{}'", ssh_command, command);

    let mut child = Command::new("sh")
        .arg("-c")
        .arg(&full_command)
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .stdin(Stdio::piped())
        .spawn()
        .map_err(|e| MaestroError(format!("Failed to execute SSH command: {}", e)))?;

    if let AuthMethod::Password(password) = &host.auth_method {
        if let Some(mut stdin) = child.stdin.take() {
            let password = password.clone();
            tokio::spawn(async move {
                stdin.write_all(format!("{}\n", password).as_bytes()).await.ok();
            });
        }
    }

    let output = child.wait_with_output().await
        .map_err(|e| MaestroError(format!("Failed to get command output: {}", e)))?;

    let stdout = String::from_utf8_lossy(&output.stdout).to_string();
    let stderr = String::from_utf8_lossy(&output.stderr).to_string();

    if output.status.success() {
        Ok(stdout)
    } else {
        Err(MaestroError(format!("Command failed: {}\nStderr: {}", stdout, stderr)))
    }
}

/// Deploys to a specific host (local or remote).
///
/// This function determines whether to deploy locally or remotely based on the host address,
/// ensures Docker is installed, and then performs the deployment.
///
/// # Arguments
///
/// * `host` - A reference to the Host struct containing host information
/// * `config` - A reference to the Config struct containing application configuration
///
/// # Returns
///
/// * `Ok(())` if the deployment is successful
/// * `Err(MaestroError)` if any step of the deployment process fails
///
/// # Errors
///
/// This function will return an error if:
/// - Docker installation fails
/// - Local or remote deployment fails
async fn deploy_to_host(host: &Host, config: &Config) -> Result<(), MaestroError> {
    match host.address.as_str() {
        "localhost" => {
            ensure_docker_installed_local().await?;
            deploy_locally(config).await
        },
        _ => {
            ensure_docker_installed_remote(host).await?;
            deploy_remotely(host, config).await
        }
    }
}

/// Main function to run the Horizon Maestro application.
///
/// This function orchestrates the entire deployment process, including:
/// 1. Reading and parsing the configuration file
/// 2. Checking for npm availability
/// 3. Starting the API server
/// 4. Building or starting the dashboard
/// 5. Deploying to all configured hosts
/// 6. Printing a deployment summary
///
/// The function keeps the application running until interrupted by a Ctrl+C signal.
///
/// # Returns
///
/// * `Ok(())` if the application runs and exits successfully
/// * `Err(MaestroError)` if any critical step fails
///
/// # Errors
///
/// This function will return an error if:
/// - The configuration file cannot be read or parsed
/// - npm is not available
/// - The API server fails to start
/// - Dashboard building or starting fails
/// - Deployment to any host fails
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

    // Deploy to all configured hosts
    for host in &config.deployment.hosts {
        match deploy_to_host(host, &config).await {
            Ok(_) => println!("{}", format!("✅ Deployment to {} successful", host.address).green().bold()),
            Err(e) => eprintln!("{}", format!("❌ Deployment to {} failed: {}", host.address, e).red().bold()),
        }
    }

    // Print deployment summary
    print_deployment_summary(&config);
    
    // Keep the application running until interrupted
    println!("\n{}", "🔄 Application is running. Press Ctrl+C to stop.".yellow().bold());
    tokio::signal::ctrl_c().await.map_err(|e| MaestroError(format!("Failed to listen for Ctrl+C: {}", e)))?;
    println!("{}", "👋 Shutting down".magenta().bold());

    Ok(())
}

/// Prints a summary of the deployment.
///
/// This function provides an overview of the deployed containers, hosts, and other relevant information.
///
/// # Arguments
///
/// * `config` - A reference to the Config struct containing application configuration
fn print_deployment_summary(config: &Config) {
    println!("{}", "📊 Deployment Summary:".cyan().bold());
    println!("   Dashboard: http://localhost:{}", DASHBOARD_PORT);
    println!("   Deployed Containers:");
    for container in &config.docker.containers {
        println!("     - Image: {}", container.image_name);
        println!("       Container Name: {}", container.container_name);
    }
    println!("   Deployed Hosts:");
    for host in &config.deployment.hosts {
        let port_info = host.ssh_port.map_or(String::new(), |port| format!(" (SSH port: {})", port));
        println!("     - {}{}", host.address, port_info);
    }
    
    println!("\n{}", "🔍 Notes:".yellow().bold());
    println!("   - Some containers may exit immediately after running if they're designed for short-lived tasks.");
    println!("   - Use 'docker ps -a' to see all containers, including stopped ones.");
    println!("   - To view container logs, use: docker logs <container_name>");
}