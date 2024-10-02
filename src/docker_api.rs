use crate::{Config, Host, ContainerConfig, MaestroError};
use tokio::process::Command;
use futures::future::join_all;
use colored::*;

/// Checks if Docker is installed on the local machine.
/// If Docker is not installed, it attempts to install it.
///
/// # Returns
/// - `Ok(())` if Docker is installed or successfully installed
/// - `Err(MaestroError)` if there was an error checking or installing Docker
pub async fn ensure_docker_installed_local() -> Result<(), MaestroError> {
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

/// Checks if Docker is installed on a remote host.
/// If Docker is not installed, it attempts to install it.
///
/// # Arguments
/// * `host` - A reference to the Host struct containing connection details
///
/// # Returns
/// - `Ok(())` if Docker is installed or successfully installed
/// - `Err(MaestroError)` if there was an error checking or installing Docker
pub async fn ensure_docker_installed_remote(host: &Host) -> Result<(), MaestroError> {
    println!("{}", format!("🐳 Checking Docker installation on {}...", host.address).blue().bold());

    let check_docker = crate::system_api::run_ssh_command("command -v docker || echo 'Docker not found'", host).await?;

    if check_docker.contains("Docker not found") {
        println!("{}", format!("⚠️  Docker is not installed on {}. Attempting to install...", host.address).yellow().bold());
        install_docker_remote(host).await
    } else {
        println!("{}", format!("✅ Docker is installed on {}", host.address).green().bold());
        Ok(())
    }
}

/// Installs Docker on the local machine.
///
/// # Returns
/// - `Ok(())` if Docker was successfully installed
/// - `Err(MaestroError)` if there was an error during installation
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
/// # Arguments
/// * `host` - A reference to the Host struct containing connection details
///
/// # Returns
/// - `Ok(())` if Docker was successfully installed
/// - `Err(MaestroError)` if there was an error during installation
async fn install_docker_remote(host: &Host) -> Result<(), MaestroError> {
    println!("{}", format!("📥 Installing Docker on {}...", host.address).blue().bold());

    let install_command = r#"
        curl -fsSL https://get.docker.com -o get-docker.sh && 
        sudo sh get-docker.sh && 
        sudo usermod -aG docker $USER && 
        echo 'Docker installed successfully'
    "#;
    
    let output = crate::system_api::run_ssh_command(install_command, host).await?;

    if output.contains("Docker installed successfully") {
        println!("{}", format!("✅ Docker installed successfully on {}", host.address).green().bold());
        Ok(())
    } else {
        Err(MaestroError(format!("Docker installation failed on {}: {}", host.address, output)))
    }
}

/// Deploys containers locally based on the provided configuration.
///
/// # Arguments
/// * `config` - A reference to the Config struct containing deployment details
///
/// # Returns
/// - `Ok(())` if all containers were successfully deployed
/// - `Err(MaestroError)` if there was an error during deployment
pub async fn deploy_locally(config: &Config) -> Result<(), MaestroError> {
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

/// Deploys containers to a remote host based on the provided configuration.
///
/// # Arguments
/// * `host` - A reference to the Host struct containing connection details
/// * `config` - A reference to the Config struct containing deployment details
///
/// # Returns
/// - `Ok(())` if all containers were successfully deployed
/// - `Err(MaestroError)` if there was an error during deployment
pub async fn deploy_remotely(host: &Host, config: &Config) -> Result<(), MaestroError> {
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

/// Deploys a single container locally.
///
/// # Arguments
/// * `container` - A reference to the ContainerConfig struct containing container details
///
/// # Returns
/// - `Ok(())` if the container was successfully deployed
/// - `Err(MaestroError)` if there was an error during deployment
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

/// Deploys a single container instance to a remote host.
///
/// # Arguments
/// * `host` - A reference to the Host struct containing connection details
/// * `container` - A reference to the ContainerConfig struct containing container details
/// * `instance` - The instance number of the container
///
/// # Returns
/// - `Ok(())` if the container instance was successfully deployed
/// - `Err(MaestroError)` if there was an error during deployment
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
        let output = crate::system_api::run_ssh_command(&cmd, host).await?;
        println!("SSH OUTPUT | {}@{}:{} / $ {}", host.username, host.address, host.ssh_port.unwrap_or(22), cmd);
        for line in output.lines() {
            println!("SSH OUTPUT | {}", line);
        }
    }

    println!("{}", format!("✅ Container '{}' (instance {}) deployed to {}", container.container_name, instance, host.address).green().bold());
    Ok(())
}