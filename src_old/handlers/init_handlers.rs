use axum::routing::get;
use socketioxide::{
    extract::{Data, SocketRef},
    SocketIo,
};
use std::net::IpAddr;
use std::str::FromStr;
use std::sync::{Arc, Mutex};
use serde_json::{Value, json};

pub struct Coordinate {
    x: f64,
    y: f64,
    z: f64,
}

impl Coordinate {
    pub fn new(x: f64, y: f64, z: f64) -> Self {
        Coordinate { x, y, z }
    }
}

pub struct IPAddress {
    address: IpAddr,
    port: u16,
}

impl IPAddress {
    pub fn new(address: IpAddr, port: u16) -> Self {
        IPAddress { address, port }
    }
    pub fn from_string(s: &str) -> Result<Self, Box<dyn std::error::Error>> {
        let parts: Vec<&str> = s.split(':').collect();
        if parts.len() != 2 {
            return Err("Invalid IP address format. Expected IP:PORT".into());
        }
        let address = IpAddr::from_str(parts[0])?;
        let port = parts[1].parse::<u16>()?;
        Ok(IPAddress { address, port })
    }
}

pub struct ChildServer {
    id: u64,
    coordinate: Coordinate,
    parent_addr: IPAddress,
    socket_ref: SocketRef,
}

impl ChildServer {
    pub fn new(
        id: u64,
        coordinate: Coordinate,
        parent_addr: IPAddress,
        socket_ref: SocketRef,
    ) -> Self {
        ChildServer {
            id,
            coordinate,
            parent_addr,
            socket_ref,
        }
    }
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let child_servers: Arc<Mutex<Vec<ChildServer>>> = Arc::new(Mutex::new(Vec::new()));
   
    let (layer, io) = SocketIo::new_layer();
   
    let child_servers_clone = child_servers.clone();
    io.ns("/", move |socket: SocketRef| {
        println!("Child server connected: {}", socket.id);
       
        let child_servers = child_servers_clone.clone();
        socket.on("authChildServer", move |s: SocketRef, d: Data<Value>| {
            handle_auth_child_server(s, d, child_servers.clone())
        });
    });

    let app = axum::Router::new()
        .route("/", get(|| async { "Horizon Maestro API" }))
        .layer(layer);
    let listener = tokio::net::TcpListener::bind("0.0.0.0:3010").await?;
    println!("Master server listening on port 3000");
    axum::serve(listener, app).await?;
    Ok(())
}

fn handle_auth_child_server(socket: SocketRef, data: Data<Value>, child_servers: Arc<Mutex<Vec<ChildServer>>>) {
    let data = data.0;  // Unwrap the Data<Value>
    if let (Some(id), Some(x), Some(y), Some(z), Some(parent_addr_str)) = (
        data.get("id").and_then(|v| v.as_u64()),
        data.get("x").and_then(|v| v.as_f64()),
        data.get("y").and_then(|v| v.as_f64()),
        data.get("z").and_then(|v| v.as_f64()),
        data.get("parentAddr").and_then(|v| v.as_str()),
    ) {
        let coordinate = Coordinate::new(x, y, z);
        if let Ok(parent_addr) = IPAddress::from_string(parent_addr_str) {
            let child_server = ChildServer::new(id, coordinate, parent_addr, socket.clone());
           
            let mut servers = child_servers.lock().unwrap();
            servers.push(child_server);
           
            println!("Child server authenticated: {}", id);
            socket.emit("authSuccess", json!({"message": "Authentication successful"})).ok();
        } else {
            socket.emit("authFailed", json!({"message": "Invalid parent address format"})).ok();
        }
    } else {
        socket.emit("authFailed", json!({"message": "Invalid authentication data"})).ok();
    }
}