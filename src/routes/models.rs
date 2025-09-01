use rocket::serde::{Serialize, Deserialize};
use std::collections::HashMap;

// Data structures
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AppInstance {
    pub id: String,
    pub name: String,
    pub image: String,
    pub status: String,
    pub created_at: String,
    pub ports: Vec<PortMapping>,
    pub environment: HashMap<String, String>,
    pub volumes: Vec<VolumeMapping>,
    pub agent_id: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PortMapping {
    pub host_port: u16,
    pub container_port: u16,
    pub protocol: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VolumeMapping {
    pub host_path: String,
    pub container_path: String,
}

#[derive(Debug, Clone, rocket::serde::Serialize, rocket::serde::Deserialize)]
#[serde(crate = "rocket::serde")]
pub struct AppInstanceRequest {
    pub name: String,
    pub image: String,
    pub ports: Option<Vec<PortMapping>>,
    pub environment: Option<HashMap<String, String>>,
    pub volumes: Option<Vec<VolumeMapping>>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VolumeInfo {
    pub name: String,
    pub mountpoint: String,
    pub labels: HashMap<String, String>,
    pub created_at: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VolumeCreateRequest {
    pub name: String,
    pub labels: Option<HashMap<String, String>>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NetworkInfo {
    pub id: String,
    pub name: String,
    pub driver: String,
    pub scope: String,
    pub containers: HashMap<String, NetworkContainerInfo>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NetworkContainerInfo {
    pub name: String,
    pub endpoint_id: String,
    pub ipv4_address: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NetworkCreateRequest {
    pub name: String,
    pub driver: Option<String>,
    pub labels: Option<HashMap<String, String>>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AgentInfo {
    pub id: String,
    pub name: String,
    pub version: String,
    pub platform: String,
    pub instance_count: usize,
    pub status: String,
    pub resources: SystemResources,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SystemResources {
    pub cpu_count: usize,
    pub memory_total: u64,
    pub memory_available: u64,
    pub disk_total: u64,
    pub disk_available: u64,
}