//==============================================================================
// Horizon Master Server - Core Implementation
//==============================================================================
// A high-performance, multithreaded master server using Socket.IO for real-time 
// communication between game servers. Features include:
//
// - Scalable thread pool architecture supporting up to 32 child game servers
// - Dynamic server connection management with automatic load balancing
// - Integrated plugin system for extensible functionality
// - Comprehensive logging and monitoring
// - Real-time Socket.IO event handling
// - Graceful error handling and connection management
//
// Structure:
// - Child server connections are distributed across multiple thread pools
// - Each pool manages up to 10 game servers independently
// - Message passing system for inter-thread communication
// - Asynchronous event handling using Tokio runtime
//
// Authors: Tristan James Poland, Thiago M. R. Goulart, Michael Houston
// License: Apache-2.0
//==============================================================================

use horizon_data_types::*;
use horizon_logger::{HorizonLogger, log_info, log_debug, log_warn, log_error, log_critical};
use serde_json::Value;
use socketioxide::extract::{Data, SocketRef};
use std::sync::{Arc, RwLock};
use std::thread;
use std::time::Instant;
use tokio::runtime::Runtime;
use tokio::sync::mpsc;
use uuid::Uuid;
use viz::{handler::ServiceHandler, serve, Body, Request, Response, Result, Router};
use once_cell::sync::Lazy;
use plugin_api;
use serde::Deserialize;
use std::fs;

mod config;
mod servers;
mod splash;

use config::Config;

//------------------------------------------------------------------------------
// Global Logger Configuration
//------------------------------------------------------------------------------

/// Global logger instance using lazy initialization
/// This ensures the logger is only created when first accessed
static CONFIG: Lazy<config::Config> = Lazy::new(|| {
    config::Config::from_file("config.yml")
});

static LOGGER: Lazy<HorizonLogger> = Lazy::new(|| {
    let logger = HorizonLogger::new();
    log_info!(logger, "INIT", "Horizon master logger initialized with level: {}", CONFIG.log_level);
    logger
});

//------------------------------------------------------------------------------
// Thread Pool Structure
//------------------------------------------------------------------------------

/// Represents a thread pool that manages a subset of connected game servers
/// Uses Arc and RwLock for safe concurrent access across threads
#[derive(Clone)]
struct ServerThreadPool {
    /// Starting index for this pool's server range
    start_index: usize,
    /// Ending index for this pool's server range
    end_index: usize,
    /// Thread-safe vector containing the game servers managed by this pool
    servers: Arc<RwLock<Vec<GameServer>>>,
    /// Channel sender for sending messages to the pool's message handler
    sender: mpsc::Sender<ServerMessage>,
    /// Thread-safe logger instance for this pool
    logger: Arc<HorizonLogger>,
}

/// Messages that can be processed by the server thread pools
enum ServerMessage {
    /// Message for adding a new game server with its socket and initial data
    NewServer(SocketRef, Value),
    /// Message for removing a game server using their UUID
    RemoveServer(Uuid),
}

//------------------------------------------------------------------------------
// Main Server Structure
//------------------------------------------------------------------------------

/// Main master server structure that manages multiple server thread pools
/// Handles incoming connections from game servers and distributes them across available pools
#[derive(Clone)]
struct HorizonMasterServer {
    // Config Values
    servers_per_pool: usize, // Number of game servers per pool
    num_thread_pools: usize, // Number of thread pools

    /// Vector of thread pools, wrapped in Arc for thread-safe sharing
    thread_pools: Arc<Vec<Arc<ServerThreadPool>>>,
    /// Tokio runtime for handling async operations
    runtime: Arc<Runtime>,
    /// Server-wide logger instance
    logger: Arc<HorizonLogger>,
}

impl HorizonMasterServer {
    /// Creates a new instance of the Horizon Master Server
    /// Initializes the thread pools and sets up message handling for each
    fn new(servers_per_pool: usize, num_thread_pools: usize) -> Self {
        let runtime = Arc::new(Runtime::new().unwrap());
        let mut thread_pools = Vec::new();
        let logger = Arc::new(HorizonLogger::new());

        log_info!(logger, "SERVER", "Initializing Horizon Master Server");
        
        // Initialize thread pools
        for i in 0..num_thread_pools {
            let start_index = i * servers_per_pool;
            let end_index = start_index + servers_per_pool;
            
            // Create message channel for this pool
            let (sender, mut receiver) = mpsc::channel(100);
            let servers = Arc::new(RwLock::new(Vec::new()));
            
            let pool = Arc::new(ServerThreadPool {
                start_index,
                end_index,
                servers: servers.clone(),
                sender,
                logger: logger.clone(),
            });

            // Initialize plugin system for this pool
            let my_manager = plugin_api::PluginManager::new();
            my_manager.load_all();

            // Spawn dedicated thread for handling this pool's messages
            let pool_clone = pool.clone();
            thread::spawn(move || {
                let rt = Runtime::new().unwrap();
                rt.block_on(async move {
                    while let Some(msg) = receiver.recv().await {
                        Self::handle_message(msg, &pool_clone).await;
                    }
                });
            });

            log_debug!(logger, "THREAD_POOL", "Initialized pool {} with range {}-{}", 
                i, start_index, end_index);
            
            thread_pools.push(pool);
        }

        HorizonMasterServer {
            servers_per_pool,
            num_thread_pools,
            thread_pools: Arc::new(thread_pools),
            runtime,
            logger,
        }
    }

    /// Handles incoming messages for a specific thread pool
    /// Processes game server connections and disconnections
    async fn handle_message(msg: ServerMessage, pool: &ServerThreadPool) {
        match msg {
            // Handle new game server connection
            ServerMessage::NewServer(socket, data) => {
                // Confirm connection to client
                socket.emit("connected", &true).ok();

                log_info!(pool.logger, "CONNECTION", "Game server {} connected successfully", 
                    socket.id.as_str());

                let id = socket.id.as_str();
                let server: GameServer = GameServer::new(socket.clone());
                
                // Initialize server-specific handlers
                servers::init(socket.clone(), pool.servers.clone());

                // Add server to pool
                pool.servers.write().unwrap().push(server.clone());

                log_debug!(pool.logger, "SERVER", "Game server {} (UUID: {}) added to pool", 
                    id, server.id);
                log_debug!(pool.logger, "SOCKET", "Socket.IO namespace: {:?}, id: {:?}", 
                    socket.ns(), socket.id);

                // Send initialization events to game server
                if let Err(e) = socket.emit("server_ready", &true) {
                    log_warn!(pool.logger, "EVENT", "Failed to emit server_ready event: {}", e);
                }
            },
            // Handle server removal
            ServerMessage::RemoveServer(server_id) => {
                let mut servers = pool.servers.write().unwrap();
                if let Some(pos) = servers.iter().position(|s| s.id == server_id) {
                    servers.remove(pos);
                    log_info!(pool.logger, "SERVER", "Game server {} removed from pool", server_id);
                } else {
                    log_warn!(pool.logger, "SERVER", "Failed to find game server {} for removal", 
                        server_id);
                }
            }
        }
    }

    /// Handles new incoming socket connections from game servers
    /// Assigns the connection to the first available thread pool
    async fn handle_new_connection(&self, socket: SocketRef, data: Data<Value>) {
        match self.thread_pools.iter().find(|pool| {
            let servers = pool.servers.read().unwrap();
            servers.len() < self.servers_per_pool
        }) {
            Some(selected_pool) => {
                log_info!(self.logger, "CONNECTION", 
                    "Assigning game server {} to thread pool {}", 
                    socket.id.to_string(), 
                    selected_pool.start_index / self.servers_per_pool);

                if let Err(e) = selected_pool.sender
                    .send(ServerMessage::NewServer(socket, data.0)).await {
                    log_error!(self.logger, "CONNECTION", 
                        "Failed to assign game server to pool: {}", e);
                }
            },
            None => {
                log_critical!(self.logger, "CAPACITY", 
                    "All thread pools are full! Cannot accept new game server connection");
            }
        }
    }

    /// Starts the master server and begins listening for game server connections
    /// Sets up Socket.IO and HTTP routing
    async fn start(self) {
        // Initialize Socket.IO service
        let (svc, io) = socketioxide::SocketIo::new_svc();
        
        let server = self.clone();
        // Configure root namespace handler
        io.ns("/", move |socket: SocketRef, data: Data<Value>| {
            let server = server.clone();
            async move {
                server.handle_new_connection(socket, data).await;
            }
        });

        // Set up HTTP routing
        let app = Router::new()
            .get("/", redirect_to_master_panel)
            .any("/*", ServiceHandler::new(svc));

        // Start server on port 3000
        match tokio::net::TcpListener::bind("0.0.0.0:3000").await {
            Ok(listener) => {
                log_info!(self.logger, "SERVER", 
                    "Master server listening on 0.0.0.0:3000");
                
                if let Err(e) = serve(listener, app).await {
                    log_critical!(self.logger, "SERVER", "Server error: {}", e);
                }
            },
            Err(e) => {
                log_critical!(self.logger, "SERVER", 
                    "Failed to bind to port 3000: {}", e);
            }
        }
    }
}

/// HTTP handler for redirecting browser access to the master panel
async fn redirect_to_master_panel(_req: Request) -> Result<Response> {
    let response = Response::builder()
        .status(302)
        .header("Location", "https://youtu.be/dQw4w9WgXcQ")
        .body(Body::empty())
        .unwrap();
    
    log_info!(LOGGER, "HTTP", "Browser access redirected to master dashboard");
    Ok(response)
}

/// Main entry point for the Horizon Master Server
#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let init_time = Instant::now();
    let servers_per_pool = CONFIG.servers_per_pool;
    let num_thread_pools = CONFIG.num_thread_pools;

    // Initialize logging system
    horizon_logger::init();
    splash::splash();
    log_info!(LOGGER, "STARTUP", "Horizon Master Server starting...");

    // Create and start server instance with configuration values
    let server = HorizonMasterServer::new(servers_per_pool, num_thread_pools);
    log_info!(LOGGER, "STARTUP", "Master server startup completed in {:?}", init_time.elapsed());
    
    server.start().await;
    
    Ok(())
}