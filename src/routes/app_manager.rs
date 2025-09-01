use std::sync::{Arc, Mutex};
use std::collections::HashMap;
use bollard::Docker;
use crate::routes::models::AppInstance;

// Docker client wrapper
pub struct AppManager {
    pub docker: Docker,
    pub instances: Arc<Mutex<HashMap<String, AppInstance>>>,
}

impl AppManager {
    pub fn new() -> Result<Self, String> {
        // Connect to Docker with default configuration
        // Works across platforms without additional config
        let docker = match Docker::connect_with_local_defaults() {
            Ok(docker) => docker,
            Err(e) => return Err(format!("Failed to connect to Docker: {}", e)),
        };
        
        Ok(AppManager {
            docker,
            instances: Arc::new(Mutex::new(HashMap::new())),
        })
    }
}