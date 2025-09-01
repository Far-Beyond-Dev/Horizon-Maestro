use rocket::get;
use rocket::serde::json::Json;
use rocket::State;
use bollard::image::ListImagesOptions;
use bollard::system::EventsOptions;
use futures::stream::StreamExt;
use crate::routes::app_manager::AppManager;

#[get("/images")]
pub async fn list_images(app_manager: &State<AppManager>) -> Json<Vec<String>> {
    let mut images = Vec::new();
    
    // List images via Docker API
    let options = Some(ListImagesOptions::<String> {
        all: false,
        ..Default::default()
    });
    
    match app_manager.docker.list_images(options).await {
        Ok(image_list) => {
            for image in image_list {
                for tag in &image.repo_tags {
                    images.push(tag.clone());
                }
            }
        },
        Err(e) => {
            eprintln!("Failed to list images: {}", e);
        }
    }
    
    Json(images)
}

#[get("/events")]
pub async fn stream_events(app_manager: &State<AppManager>) -> String {
    // This would typically be implemented with Server-Sent Events or WebSockets
    // For this example, we'll just demonstrate the Docker events API
    
    let options = Some(EventsOptions::<String> {
        ..Default::default()
    });
    
    let mut event_stream = app_manager.docker.events(options);
    
    // In a real implementation, you'd stream these to the client
    // Here we'll just return a message
    while let Some(event) = event_stream.next().await {
        match event {
            Ok(event) => {
                println!("Event: {:?}", event);
                // In a real implementation, send this to the client
            },
            Err(e) => {
                eprintln!("Error receiving event: {}", e);
                break;
            }
        }
    }
    
    "Event streaming would happen here".to_string()
}