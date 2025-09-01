use rocket::get;
use rocket::serde::json::Json;
use rocket::State;
use uuid;
use hostname;
use num_cpus;
use sys_info;
use crate::routes::app_manager::AppManager;
use crate::routes::models::{AgentInfo, SystemResources};

// Agent Management Routes

#[get("/agent/info")]
pub async fn get_agent_info(app_manager: &State<AppManager>) -> Json<AgentInfo> {
    // Get Docker engine info
    let info = match app_manager.docker.info().await {
        Ok(info) => info,
        Err(e) => {
            eprintln!("Failed to get Docker info: {}", e);
            return Json(AgentInfo {
                id: uuid::Uuid::new_v4().to_string(),
                name: hostname::get().unwrap_or_default().to_string_lossy().to_string(),
                version: "unknown".to_string(),
                platform: "unknown".to_string(),
                instance_count: app_manager.instances.lock().unwrap().len(),
                status: "degraded".to_string(),
                resources: SystemResources {
                    cpu_count: num_cpus::get(),
                    memory_total: 0,
                    memory_available: 0,
                    disk_total: 0,
                    disk_available: 0,
                },
            });
        }
    };
    
    // Get system resources
    let memory_info = sys_info::mem_info().unwrap_or(sys_info::MemInfo {
        total: 0,
        free: 0,
        avail: 0,
        buffers: 0,
        cached: 0,
        swap_total: 0,
        swap_free: 0,
    });
    
    let disk_info = sys_info::disk_info().unwrap_or(sys_info::DiskInfo {
        total: 0,
        free: 0,
    });
    
    Json(AgentInfo {
        id: uuid::Uuid::new_v4().to_string(),
        name: hostname::get().unwrap_or_default().to_string_lossy().to_string(),
        version: info.server_version.unwrap_or_default(),
        platform: format!("{} / {}", 
            info.operating_system.unwrap_or_default(),
            info.architecture.unwrap_or_default()),
        instance_count: app_manager.instances.lock().unwrap().len(),
        status: "healthy".to_string(),
        resources: SystemResources {
            cpu_count: num_cpus::get(),
            memory_total: memory_info.total * 1024,
            memory_available: memory_info.avail * 1024,
            disk_total: disk_info.total * 1024,
            disk_available: disk_info.free * 1024,
        },
    })
}

#[get("/health")]
pub fn health_check() -> String {
    "App Manager is healthy".to_string()
}