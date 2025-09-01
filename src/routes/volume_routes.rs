use rocket::{delete, get, post};
use rocket::serde::json::Json;
use rocket::State;
use crate::routes::app_manager::AppManager;
use crate::routes::models::{VolumeInfo, VolumeCreateRequest};

// Volume Management

#[get("/volumes")]
pub async fn list_volumes(app_manager: &State<AppManager>) -> Result<Json<Vec<VolumeInfo>>, String> {
    match app_manager.docker.list_volumes::<String>(None).await {
        Ok(volumes) => {
            let volume_list = volumes.volumes.unwrap_or_default().into_iter()
                .filter_map(|vol| {
                    let name = vol.name;
                    let mountpoint = vol.mountpoint;
                    let labels = vol.labels;
                    let created_at = vol.created_at.unwrap_or_default();
                    
                    Some(VolumeInfo {
                        name,
                        mountpoint,
                        labels,
                        created_at,
                    })
                })
                .collect();
            
            Ok(Json(volume_list))
        },
        Err(e) => Err(format!("Failed to list volumes: {}", e))
    }
}

#[post("/volumes", format = "json", data = "<volume_req>")]
pub async fn create_volume(volume_req: Json<VolumeCreateRequest>, app_manager: &State<AppManager>) -> Result<Json<VolumeInfo>, String> {
    let options = bollard::volume::CreateVolumeOptions {
        name: volume_req.name.clone(),
        labels: volume_req.labels.clone().unwrap_or_default(),
        ..Default::default()
    };
    
    match app_manager.docker.create_volume(options).await {
        Ok(volume) => {
            let volume_info = VolumeInfo {
                name: volume.name,
                mountpoint: volume.mountpoint,
                labels: volume.labels,
                created_at: volume.created_at.unwrap_or_default(),
            };
            
            Ok(Json(volume_info))
        },
        Err(e) => Err(format!("Failed to create volume: {}", e))
    }
}

#[delete("/volumes/<name>")]
pub async fn delete_volume(name: String, app_manager: &State<AppManager>) -> Result<String, String> {
    match app_manager.docker.remove_volume(&name, None).await {
        Ok(_) => Ok(format!("Volume {} deleted successfully", name)),
        Err(e) => Err(format!("Failed to delete volume: {}", e))
    }
}