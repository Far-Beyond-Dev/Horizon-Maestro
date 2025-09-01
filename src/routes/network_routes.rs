use rocket::{delete, get, post, put};
use rocket::serde::json::Json;
use rocket::State;
use std::collections::HashMap;
use crate::routes::app_manager::AppManager;
use crate::routes::models::{NetworkInfo, NetworkCreateRequest, NetworkContainerInfo};

// Network Management

#[get("/networks")]
pub async fn list_networks(app_manager: &State<AppManager>) -> Result<Json<Vec<NetworkInfo>>, String> {
    match app_manager.docker.list_networks::<String>(None).await {
        Ok(networks) => {
            let network_list = networks.into_iter()
                .filter_map(|net| {
                    let id = net.id?;
                    let name = net.name?;
                    let driver = net.driver?;
                    let scope = net.scope?;
                    
                    let mut containers = HashMap::new();
                    if let Some(net_containers) = net.containers {
                        for (container_id, container_info) in net_containers {
                            if let (Some(name), Some(endpoint_id), Some(ipv4_address)) = 
                               (container_info.name, container_info.endpoint_id, container_info.ipv4_address) {
                                containers.insert(container_id, NetworkContainerInfo {
                                    name,
                                    endpoint_id,
                                    ipv4_address,
                                });
                            }
                        }
                    }
                    
                    Some(NetworkInfo {
                        id,
                        name,
                        driver,
                        scope,
                        containers,
                    })
                })
                .collect();
            
            Ok(Json(network_list))
        },
        Err(e) => Err(format!("Failed to list networks: {}", e))
    }
}

#[post("/networks", format = "json", data = "<network_req>")]
pub async fn create_network(network_req: Json<NetworkCreateRequest>, app_manager: &State<AppManager>) -> Result<Json<NetworkInfo>, String> {
    let options = bollard::network::CreateNetworkOptions {
        name: network_req.name.clone(),
        driver: network_req.driver.clone().unwrap_or_default(),
        labels: network_req.labels.clone().unwrap_or_default(),
        ..Default::default()
    };
    
    match app_manager.docker.create_network(options).await {
        Ok(response) => {
            // Inspect network to get full details
            match app_manager.docker.inspect_network::<String>(response.id.as_str(), None).await {
                Ok(network) => {
                    let mut containers = HashMap::new();
                    if let Some(net_containers) = network.containers {
                        for (container_id, container_info) in net_containers {
                            if let (Some(name), Some(endpoint_id), Some(ipv4_address)) = 
                               (container_info.name, container_info.endpoint_id, container_info.ipv4_address) {
                                containers.insert(container_id, NetworkContainerInfo {
                                    name,
                                    endpoint_id,
                                    ipv4_address,
                                });
                            }
                        }
                    }
                    
                    let network_info = NetworkInfo {
                        id: network.id.unwrap_or_default(),
                        name: network.name.unwrap_or_default(),
                        driver: network.driver.unwrap_or_default(),
                        scope: network.scope.unwrap_or_default(),
                        containers,
                    };
                    
                    Ok(Json(network_info))
                },
                Err(e) => Err(format!("Failed to inspect created network: {}", e))
            }
        },
        Err(e) => Err(format!("Failed to create network: {}", e))
    }
}

#[delete("/networks/<id>")]
pub async fn delete_network(id: String, app_manager: &State<AppManager>) -> Result<String, String> {
    match app_manager.docker.remove_network(&id).await {
        Ok(_) => Ok(format!("Network {} deleted successfully", id)),
        Err(e) => Err(format!("Failed to delete network: {}", e))
    }
}

#[put("/instances/<id>/connect/<network_id>")]
pub async fn connect_instance_to_network(id: String, network_id: String, app_manager: &State<AppManager>) -> Result<String, String> {
    let options = bollard::network::ConnectNetworkOptions {
        container: id.clone(),
        ..Default::default()
    };
    
    match app_manager.docker.connect_network(&network_id, options).await {
        Ok(_) => Ok(format!("Instance {} connected to network {}", id, network_id)),
        Err(e) => Err(format!("Failed to connect instance to network: {}", e))
    }
}

#[put("/instances/<id>/disconnect/<network_id>")]
pub async fn disconnect_instance_from_network(id: String, network_id: String, app_manager: &State<AppManager>) -> Result<String, String> {
    let options = bollard::network::DisconnectNetworkOptions {
        container: id.clone(),
        force: false,
    };
    
    match app_manager.docker.disconnect_network(&network_id, options).await {
        Ok(_) => Ok(format!("Instance {} disconnected from network {}", id, network_id)),
        Err(e) => Err(format!("Failed to disconnect instance from network: {}", e))
    }
}