![Horizon Maestro Splash](branding/logo-no-background.png)
# Horizon Maestro (Rust Version)

Horizon Maestro is the master control system for the Horizon Game server, now implemented in Rust for improved performance and safety. It orchestrates the deployment, management, and monitoring of game server instances, ensuring smooth operation and optimal performance of the Horizon gaming platform.

## Overview

Horizon Maestro acts as the central management hub for the Horizon Game server ecosystem. Built with Rust, it provides a high-performance, memory-safe, and concurrent solution for automating server deployment, managing server resources, monitoring game performance, and coordinating player sessions across multiple server instances.

## Key Features

1. **Asynchronous Server Management**: Utilizes Rust's async/await syntax for efficient, non-blocking operations.
2. **Docker Integration**: Manages game server containers using the Docker API.
3. **Configuration Management**: Uses TOML for flexible and readable configuration.
4. **NPM Integration**: Runs npm scripts for the Horizon Dashboard.
5. **Socket Communication**: Implements both a socket server and client for inter-process communication.
6. **Error Handling**: Leverages Rust's robust error handling for increased reliability.

## Architecture

Horizon Maestro is built using a modular, asynchronous architecture:

- **Main Loop**: Coordinated by Tokio runtime for concurrent task management.
- **Config Parser**: Uses `serde` and `toml` for parsing configuration files.
- **Docker Client**: Implemented with `reqwest` for API communication.
- **Socket Server**: Listens on port 3010 for incoming connections.
- **Socket Client**: Connects to port 3011 for outgoing communications.
- **NPM Runner**: Executes npm scripts in a separate process.

## Getting Started

To set up Horizon Maestro:

1. Ensure you have Rust and Cargo installed (https://www.rust-lang.org/tools/install)
2. Clone the Maestro repository
3. Navigate to the project directory
4. Build the project: `cargo build --release`
5. Configure your settings in `config.toml`
6. Run Maestro: `cargo run --release`

## Configuration

Maestro is configured using a TOML file (`config.toml`). Key configuration options include:

```toml
[docker]
base_url = "http://localhost:2375"
image_name = "horizon-game-server:latest"
container_name = "horizon-instance-1"
```

## Dependencies

Horizon Maestro relies on the following Rust crates:

- `tokio`: Asynchronous runtime
- `serde` and `serde_derive`: Serialization and deserialization
- `toml`: Configuration file parsing
- `reqwest`: HTTP client for Docker API communication

Add these to your `Cargo.toml`:

```toml
[dependencies]
tokio = { version = "1.0", features = ["full"] }
serde = "1.0"
serde_derive = "1.0"
serde_json = "1.0"
toml = "0.5"
reqwest = { version = "0.11", features = ["json"] }
```

## Core Components

### NPM Start Runner

```rust
async fn run_npm_start() -> Result<(), Box<dyn std::error::Error>> {
    // Implementation details...
}
```

Executes the `npm run start` command in the Horizon-Dashboard directory.

### Docker Container Creator

```rust
async fn create_docker_container(config: &Config) -> Result<(), Box<dyn std::error::Error>> {
    // Implementation details...
}
```

Creates a Docker container based on the configuration settings.

### Socket Server

```rust
async fn run_socket_server() -> Result<(), Box<dyn std::error::Error>> {
    // Implementation details...
}
```

Runs a socket server on port 3010, handling incoming connections.

### Socket Client

```rust
async fn run_socket_client() -> Result<(), Box<dyn std::error::Error>> {
    // Implementation details...
}
```

Connects to a socket server on port 3011 for outgoing communications.

## Error Handling

Maestro uses Rust's `Result` type for comprehensive error handling. All errors are propagated to the main function and logged appropriately.

## Contributing

We welcome contributions to Horizon Maestro! Please see our [Contributing Guide](./CONTRIBUTING.md) for more information on how to get involved.

## License

Horizon Maestro is licensed under the [MIT License](./LICENSE).

## Support

For support, please open an issue on the GitHub repository or contact our support team at support@horizongame.com.
