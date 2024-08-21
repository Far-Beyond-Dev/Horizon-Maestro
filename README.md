# Horizon Maestro

Horizon Maestro is a specialized deployment tool designed to streamline the process of deploying and scaling Horizon Game server services and their dependencies. It automates the deployment of Docker containers across multiple hosts, ensuring efficient and consistent setup of game server infrastructure.

## Features

- 🎮 Automated deployment of Horizon Game server services
- 🚀 Scaling capabilities for game server dependencies
- 🐳 Docker container deployment across multiple hosts (local and remote)
- 🔧 Automatic Docker installation if not present on target hosts
- 🔐 Support for both password and SSH key authentication for remote hosts
- 📊 Detailed deployment summary and status reporting for game server infrastructure
- 🖥️ Dashboard for monitoring and managing game server deployments

## Prerequisites

- Rust (latest stable version)
- npm (for building and running the monitoring dashboard)
- SSH access to remote hosts (for distributed deployment)
- Docker (will be automatically installed if not present)

## Installation

1. Clone the repository:
   ```
   git clone https://github.com/your-username/horizon-maestro.git
   cd horizon-maestro
   ```

2. Build the project:
   ```
   cargo build --release
   ```

## Configuration

Create a `config.toml` file in the project root with the following structure:

```toml
[npm]
dashboard_path = "./path/to/monitoring/dashboard"

[deployment]
hosts = [
  { address = "localhost", username = "local_user", auth_method = { Password = "local_password" } },
  { address = "192.168.1.100", username = "game_server1", auth_method = { Password = "server1_password" } },
  { address = "192.168.1.101", username = "game_server2", auth_method = { Key = "/path/to/ssh_key" } }
]

[docker]
image_name = "horizon-game-server:latest"
container_name = "horizon-game-server"
```

Adjust the values according to your game server setup and infrastructure.

## Usage

Run the application:

```
cargo run --release
```

Horizon Maestro will:
1. Check for npm and Docker installations on all specified hosts
2. Build and start the monitoring dashboard
3. Deploy Horizon Game server containers to all specified hosts
4. Deploy and scale any necessary dependencies for the game servers
5. Provide a summary of the game server infrastructure deployment

## Game Server Management

- Use the monitoring dashboard to view the status of your game servers and their dependencies.
- Scale your game server infrastructure by adding new hosts to the `config.toml` file and re-running Maestro.
- For specific game server logs, use: `docker logs horizon-game-server` on the respective host.

## Support

For issues specific to Horizon Game server deployment or scaling, please open an issue on the GitHub repository or contact the Horizon game server support team.