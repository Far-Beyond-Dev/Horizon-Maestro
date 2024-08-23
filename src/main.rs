    //! Horizon Maestro: A deployment and management tool for distributed systems
    //!
    //! This module provides functionality to deploy and manage containers across
    //! multiple hosts, both locally and remotely. It also includes a dashboard
    //! for monitoring and control.

    use std::path::{Path, PathBuf};
    use std::sync::Arc;
    use std::fs;
    use std::process::Stdio;
    use serde_derive::{Deserialize, Serialize};
    use tokio::process::Command;
    use tokio::io::AsyncWriteExt;
    use tokio::sync::Mutex;
    use std::error::Error;
    use std::fmt;
    use colored::*;
    use tokio::sync::oneshot;

    mod api;

    /// Custom error type for Maestro operations
    #[derive(Debug)]
    struct MaestroError(String);

    impl fmt::Display for MaestroError {
        fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
            write!(f, "{}", self.0)
        }
    }

    impl Error for MaestroError {}

    /// Main configuration structure for Maestro
    #[derive(Deserialize, Serialize, Clone)]
    struct Config {
        npm: NpmConfig,
        deployment: DeploymentConfig,
        docker: DockerConfig,
    }

    /// Configuration for NPM-related operations
    #[derive(Deserialize, Serialize, Clone)]
    struct NpmConfig {
        dashboard_path: String,
    }

    /// Configuration for deployment hosts
    #[derive(Deserialize, Serialize, Clone)]
    struct DeploymentConfig {
        hosts: Vec<Host>,
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
    }

    /// Configuration for individual Docker containers
    #[derive(Deserialize, Serialize, Clone)]
    struct ContainerConfig {
        image_name: String,
        container_name: String,
    }

    const BUILT_INDICATOR: &str = ".dashboard_built";
    const DASHBOARD_PORT: u16 = 3007;

    /// Builds or starts the dashboard based on the configuration
    async fn build_or_start_dashboard(config: &Config) -> Result<(), MaestroError> {
        let dashboard_path = PathBuf::from(&config.npm.dashboard_path);
    
        if !dashboard_path.exists() {
            return Err(MaestroError(format!("Dashboard directory not found at {:?}", dashboard_path)));
        }

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

    /// Runs an npm command in the specified directory
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

    /// Checks if npm is available on the system
    async fn is_npm_available() -> bool {
        Command::new("npm")
            .arg("--version")
            .output()
            .await
            .is_ok()
    }

    /// Deploys containers locally using Docker
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

    /// Deploys containers to a remote host
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

    /// Ensures Docker is installed on a remote host
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

    /// Ensures Docker is installed locally
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

    /// Installs Docker locally
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

    /// Installs Docker on a remote host
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

    /// Builds the SSH command string based on the host configuration
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

    /// Runs an SSH command and handles password input if necessary
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

    /// Deploys to a specific host (local or remote)
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

    /// Deploys to all hosts specified in the configuration
    async fn deploy_to_all_hosts(config: &Config) -> Result<(), MaestroError> {
        for host in &config.deployment.hosts {
            deploy_to_host(host, config).await?;
        }
        Ok(())
    }

    /// Main function to run the Horizon Maestro application
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
        let (shutdown_tx, shutdown_rx) = oneshot::channel();
           
        // Start the API server in a new thread
        let api_handle = tokio::spawn(async move {
            if let Err(e) = api::run_api_server(shutdown_rx).await {
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