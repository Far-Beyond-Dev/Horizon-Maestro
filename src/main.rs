use std::path::{Path, PathBuf};
use std::fs;
use serde_derive::{Deserialize, Serialize};
use tokio::process::Command;
use std::error::Error;
use std::fmt;
use colored::*;

#[derive(Debug)]
struct MaestroError(String);

impl fmt::Display for MaestroError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}

impl Error for MaestroError {}

#[derive(Deserialize, Serialize)]
struct Config {
    npm: NpmConfig,
    deployment: DeploymentConfig,
    docker: DockerConfig,
}

#[derive(Deserialize, Serialize)]
struct NpmConfig {
    dashboard_path: String,
}

#[derive(Deserialize, Serialize)]
struct DeploymentConfig {
    hosts: Vec<Host>,
}

#[derive(Deserialize, Serialize)]
struct Host {
    address: String,
    username: String,
    auth_method: AuthMethod,
}

#[derive(Deserialize, Serialize)]
enum AuthMethod {
    Password(String),
    Key(String),
}

#[derive(Deserialize, Serialize)]
struct DockerConfig {
    image_name: String,
    container_name: String,
}

const BUILT_INDICATOR: &str = ".dashboard_built";
const DASHBOARD_PORT: u16 = 3007;

async fn build_or_start_dashboard(config: &Config) -> Result<(), MaestroError> {
    let dashboard_path = PathBuf::from(&config.npm.dashboard_path);
  
    if !dashboard_path.exists() {
        return Err(MaestroError(format!("Dashboard directory not found at {:?}", dashboard_path)));
    }

    println!("{}", "📦 Installing npm dependencies...".yellow().bold());
    run_npm_command(&dashboard_path, &["install"]).await?;
    println!("{}", "✅ npm dependencies installed successfully.".green().bold());

    let built_indicator = dashboard_path.join(BUILT_INDICATOR);

    if !built_indicator.exists() {
        println!("{}", "🏗️  Building dashboard for the first time...".yellow().bold());
        run_npm_command(&dashboard_path, &["run", "build"]).await?;
        fs::File::create(&built_indicator)
            .map_err(|e| MaestroError(format!("Failed to create built indicator file: {}", e)))?;
        println!("{}", "✅ Dashboard built successfully.".green().bold());
    }

    println!("{}", "🚀 Starting dashboard preview...".cyan().bold());
    tokio::spawn(async move {
        if let Err(e) = run_npm_command(&dashboard_path, &["run", "preview", "--", "--port", &DASHBOARD_PORT.to_string()]).await {
            eprintln!("{}", format!("❌ Error starting dashboard preview: {}", e).red().bold());
        }
    });

    println!("{}", format!("🌐 Dashboard is now running at http://localhost:{}", DASHBOARD_PORT).green().bold());
    Ok(())
}

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

async fn is_npm_available() -> bool {
    Command::new("npm")
        .arg("--version")
        .output()
        .await
        .is_ok()
}

async fn deploy_locally(config: &Config) -> Result<(), MaestroError> {
    println!("{}", "🏠 Deploying locally".blue().bold());

    // Pull the Docker image
    let docker_pull = Command::new("docker")
        .args(&["pull", &config.docker.image_name])
        .output()
        .await
        .map_err(|e| MaestroError(format!("Failed to pull Docker image: {}", e)))?;

    if !docker_pull.status.success() {
        let error = String::from_utf8_lossy(&docker_pull.stderr);
        return Err(MaestroError(format!("Failed to pull Docker image: {}", error)));
    }

    // Remove existing container if it exists
    let _ = Command::new("docker")
        .args(&["rm", "-f", &config.docker.container_name])
        .output()
        .await;

    // Run the Docker container
    let docker_run = Command::new("docker")
        .args(&[
            "run",
            "-d",
            "--name", &config.docker.container_name,
            &config.docker.image_name
        ])
        .output()
        .await
        .map_err(|e| MaestroError(format!("Failed to run Docker container: {}", e)))?;

    if !docker_run.status.success() {
        let error = String::from_utf8_lossy(&docker_run.stderr);
        return Err(MaestroError(format!("Failed to run Docker container: {}", error)));
    }

    // Verify the container is running
    let docker_ps = Command::new("docker")
        .args(&["ps", "--filter", &format!("name={}", config.docker.container_name), "--format", "{{.Names}}"])
        .output()
        .await
        .map_err(|e| MaestroError(format!("Failed to verify container: {}", e)))?;

    let container_name = String::from_utf8_lossy(&docker_ps.stdout).trim().to_string();
    if container_name == config.docker.container_name {
        println!("{}", format!("✅ Container '{}' is running", config.docker.container_name).green().bold());
    } else {
        return Err(MaestroError(format!("Container '{}' is not running", config.docker.container_name)));
    }

    println!("{}", "✅ Deployed locally".green().bold());
    Ok(())
}

async fn deploy_remotely(host: &Host, config: &Config) -> Result<(), MaestroError> {
    println!("{}", format!("🌐 Deploying to {}", host.address).blue().bold());

    let ssh_command = match &host.auth_method {
        AuthMethod::Password(password) => format!(
            "sshpass -p '{}' ssh {}@{}",
            password, host.username, host.address
        ),
        AuthMethod::Key(key_path) => format!(
            "ssh -i {} {}@{}",
            key_path, host.username, host.address
        ),
    };

    let docker_commands = vec![
        format!("docker pull {}", config.docker.image_name),
        format!("docker rm -f {}", config.docker.container_name),
        format!(
            "docker run -d --name {} {}",
            config.docker.container_name, config.docker.image_name
        ),
        format!(
            "docker ps --filter name={} --format '{{{{.Names}}}}'",
            config.docker.container_name
        ),
    ];

    for cmd in docker_commands {
        let full_command = format!("{} '{}'", ssh_command, cmd);
        let output = Command::new("sh")
            .arg("-c")
            .arg(&full_command)
            .output()
            .await
            .map_err(|e| MaestroError(format!("Failed to execute command: {}", e)))?;

        println!("Command output: {}", String::from_utf8_lossy(&output.stdout));

        if !output.status.success() {
            let error = String::from_utf8_lossy(&output.stderr);
            return Err(MaestroError(format!("Command failed: {}", error)));
        }
    }

    println!("{}", format!("✅ Deployed to {}", host.address).green().bold());
    Ok(())
}

async fn ensure_docker_installed_remote(host: &Host) -> Result<(), MaestroError> {
    println!("{}", format!("🐳 Checking Docker installation on {}...", host.address).blue().bold());

    let ssh_command = match &host.auth_method {
        AuthMethod::Password(password) => format!(
            "sshpass -p '{}' ssh {}@{} 'docker --version'",
            password, host.username, host.address
        ),
        AuthMethod::Key(key_path) => format!(
            "ssh -i {} {}@{} 'docker --version'",
            key_path, host.username, host.address
        ),
    };

    let output = Command::new("sh")
        .arg("-c")
        .arg(&ssh_command)
        .output()
        .await
        .map_err(|e| MaestroError(format!("Failed to check Docker on {}: {}", host.address, e)))?;

    if output.status.success() {
        let version = String::from_utf8_lossy(&output.stdout);
        println!("{}", format!("✅ Docker is installed on {}: {}", host.address, version.trim()).green().bold());
        Ok(())
    } else {
        println!("{}", format!("⚠️  Docker is not installed on {}. Attempting to install...", host.address).yellow().bold());
        install_docker_remote(host).await
    }
}

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

async fn install_docker_remote(host: &Host) -> Result<(), MaestroError> {
    println!("{}", format!("📥 Installing Docker on {}...", host.address).blue().bold());

    let install_command = r#"curl -fsSL https://get.docker.com -o get-docker.sh && sudo sh get-docker.sh"#;
    
    let ssh_command = match &host.auth_method {
        AuthMethod::Password(password) => format!(
            "sshpass -p '{}' ssh {}@{} '{}'",
            password, host.username, host.address, install_command
        ),
        AuthMethod::Key(key_path) => format!(
            "ssh -i {} {}@{} '{}'",
            key_path, host.username, host.address, install_command
        ),
    };

    let output = Command::new("sh")
        .arg("-c")
        .arg(&ssh_command)
        .output()
        .await
        .map_err(|e| MaestroError(format!("Failed to install Docker on {}: {}", host.address, e)))?;

    if output.status.success() {
        println!("{}", format!("✅ Docker installed successfully on {}", host.address).green().bold());
        Ok(())
    } else {
        let error = String::from_utf8_lossy(&output.stderr);
        Err(MaestroError(format!("Docker installation failed on {}: {}", host.address, error)))
    }
}

async fn deploy_to_host(host: &Host, config: &Config) -> Result<(), MaestroError> {
    match host.address.as_str() {
        "localhost" => {
            ensure_docker_installed_local().await?;
            deploy_locally(config).await
        },
        _ => {
            ensure_docker_installed_remote(host).await?;
            deploy_remotely(host, config).await
        },
    }
}

async fn deploy_to_all_hosts(config: &Config) -> Result<(), MaestroError> {
    for host in &config.deployment.hosts {
        deploy_to_host(host, config).await?;
    }
    Ok(())
}

#[tokio::main]
async fn main() -> Result<(), MaestroError> {
    println!("{}", "🎭 Starting Horizon Maestro".magenta().bold());

    let config_str = fs::read_to_string("config.toml")
        .map_err(|e| MaestroError(format!("Failed to read config file: {}", e)))?;
    let config: Config = toml::from_str(&config_str)
        .map_err(|e| MaestroError(format!("Failed to parse config file: {}", e)))?;

    if is_npm_available().await {
        println!("{}", "🎉 npm found!".green().bold());
    } else {
        return Err(MaestroError("❌ npm is not available. Please install Node.js and npm.".to_string()));
    }

    build_or_start_dashboard(&config).await?;
    deploy_to_all_hosts(&config).await?;

    println!("{}", "📊 Deployment Summary:".cyan().bold());
    println!("   Dashboard: http://localhost:{}", DASHBOARD_PORT);
    println!("   Docker Image: {}", config.docker.image_name);
    println!("   Container Name: {}", config.docker.container_name);
    println!("   Deployed Hosts:");
    for host in &config.deployment.hosts {
        println!("     - {}", host.address);
    }
    
    println!("\n{}", "🔍 Notes:".yellow().bold());
    println!("   - For 'hello-world' like images, containers may exit immediately after running.");
    println!("   - Use 'docker ps -a' to see all containers, including stopped ones.");
    println!("   - To view container logs, use: docker logs {}", config.docker.container_name);
    
    // Keep the application running
    println!("\n{}", "🔄 Application is running. Press Ctrl+C to stop.".yellow().bold());
    tokio::signal::ctrl_c().await.map_err(|e| MaestroError(format!("Failed to listen for Ctrl+C: {}", e)))?;
    println!("{}", "👋 Shutting down".magenta().bold());

    Ok(())
}