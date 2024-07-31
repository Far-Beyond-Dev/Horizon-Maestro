use std::path::{Path, PathBuf};
use std::fs;
use serde_derive::Deserialize;
use reqwest;
use tokio::process::Command;
use tokio::net::{TcpListener, TcpStream};
use tokio::io::{AsyncReadExt, AsyncWriteExt};
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

#[derive(Deserialize)]
struct Config {
    docker: DockerConfig,
    npm: NpmConfig,
}

#[derive(Deserialize)]
struct DockerConfig {
    base_url: String,
    image_name: String,
    container_name: String,
}

#[derive(Deserialize)]
struct NpmConfig {
    dashboard_path: String,
}

const BUILT_INDICATOR: &str = ".dashboard_built";
const DASHBOARD_PORT: u16 = 3007;

async fn build_or_start_dashboard(config: &Config) -> Result<(), MaestroError> {
  let dashboard_path = PathBuf::from(&config.npm.dashboard_path);
  
  if !dashboard_path.exists() {
      return Err(MaestroError(format!("Dashboard directory not found at {:?}", dashboard_path)));
  }

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


async fn create_docker_container(config: &Config) -> Result<(), MaestroError> {
    let client = reqwest::Client::new();
    let create_container_endpoint = format!("{}/containers/create", config.docker.base_url);
    let container_config = serde_json::json!({
        "Image": config.docker.image_name,
        "name": config.docker.container_name,
    });

    let response = client.post(&create_container_endpoint)
        .json(&container_config)
        .send()
        .await
        .map_err(|e| MaestroError(format!("Failed to send request to Docker API: {}", e)))?;

    if response.status().is_success() {
        println!("{}", "🐳 Container created successfully.".green().bold());
        Ok(())
    } else {
        Err(MaestroError(format!("Failed to create container: {}", response.status())))
    }
}

async fn handle_client(mut socket: TcpStream) -> Result<(), MaestroError> {
    let mut buffer = [0; 1024];
    let bytes_read = socket.read(&mut buffer).await
        .map_err(|e| MaestroError(format!("Failed to read from socket: {}", e)))?;
    let received = String::from_utf8_lossy(&buffer[..bytes_read]);
    println!("{}", format!("📥 Received: {}", received).blue());

    // Process the received data here

    let response = "Processed your request";
    socket.write_all(response.as_bytes()).await
        .map_err(|e| MaestroError(format!("Failed to write to socket: {}", e)))?;
    println!("{}", "📤 Sent response".blue());
    Ok(())
}

async fn run_socket_server() -> Result<(), MaestroError> {
    let listener = TcpListener::bind("127.0.0.1:3010").await
        .map_err(|e| MaestroError(format!("Failed to bind to port 3010: {}", e)))?;
    println!("{}", "🎧 Socket server listening on port 3010".yellow().bold());

    loop {
        let (socket, _) = listener.accept().await
            .map_err(|e| MaestroError(format!("Failed to accept connection: {}", e)))?;
        tokio::spawn(async move {
            if let Err(e) = handle_client(socket).await {
                eprintln!("{}", format!("❌ Error handling client: {}", e).red().bold());
            }
        });
    }
}

async fn run_socket_client() -> Result<(), MaestroError> {
    let mut stream = TcpStream::connect("127.0.0.1:3011").await
        .map_err(|e| MaestroError(format!("Failed to connect to port 3011: {}", e)))?;
    println!("{}", "🔌 Connected to socket server on port 3011".yellow().bold());

    let message = "Hello from client";
    stream.write_all(message.as_bytes()).await
        .map_err(|e| MaestroError(format!("Failed to write to socket: {}", e)))?;

    let mut buffer = [0; 1024];
    let bytes_read = stream.read(&mut buffer).await
        .map_err(|e| MaestroError(format!("Failed to read from socket: {}", e)))?;
    let response = String::from_utf8_lossy(&buffer[..bytes_read]);
    println!("{}", format!("📥 Received response: {}", response).blue());

    Ok(())
}

async fn is_npm_available() -> bool {
    Command::new("npm")
        .arg("--version")
        .output()
        .await.is_ok()
}

#[tokio::main]
async fn main() -> Result<(), MaestroError> {
    println!("{}", "🎭 Starting Horizon Maestro".magenta().bold());

    let config_str = fs::read_to_string("config.toml")
        .map_err(|e| MaestroError(format!("Failed to read config file: {}", e)))?;
    let config: Config = toml::from_str(&config_str)
        .map_err(|e| MaestroError(format!("Failed to parse config file: {}", e)))?;

    if is_npm_available().await {
      eprintln!("{}", format!("🎉 npm found!").green().bold())
    } else {
      return Err(MaestroError("❌ npm is not available. Please install Node.js and npm.".to_string()));
    }

    build_or_start_dashboard(&config).await?;
    // create_docker_container(&config).await?;

    let server = tokio::spawn(async {
        if let Err(e) = run_socket_server().await {
            eprintln!("{}", format!("❌ Socket server error: {}", e).red().bold());
        }
    });

    //let client = tokio::spawn(async {
    //    if let Err(e) = run_socket_client().await {
    //        eprintln!("{}", format!("❌ Socket client error: {}", e).red().bold());
    //    }
    //});

    // Wait for both the server and client to complete (or for Ctrl+C)
    tokio::select! {
        _ = server => {},
        // _ = client => {},
        _ = tokio::signal::ctrl_c() => {
            println!("{}", "👋 Shutting down".magenta().bold());
        }
    }

    Ok(())
}