use actix_web::{get, web, Responder};
use crate::api::structs::*;

/// Handles GET requests for task execution history.
///
/// This endpoint provides a history of executed maintenance tasks.
#[get("/maintenance/tasks/history")]
async fn task_history() -> impl Responder {
    let history = vec![
        TaskHistory { id: 1, name: "Daily Backup".to_string(), execution_time: "2024-08-09 00:00".to_string(), status: "Completed".to_string(), duration: "15m 30s".to_string() },
        TaskHistory { id: 2, name: "Log Rotation".to_string(), execution_time: "2024-08-08 01:00".to_string(), status: "Completed".to_string(), duration: "5m 45s".to_string() },
        TaskHistory { id: 3, name: "Security Scan".to_string(), execution_time: "2024-08-07 02:00".to_string(), status: "Failed".to_string(), duration: "30m 0s".to_string() },
    ];
    web::Json(history)
}

/// Handles GET requests for scheduled maintenance tasks.
///
/// This endpoint provides information about scheduled system maintenance tasks.
#[get("/maintenance/tasks/schedule")]
async fn scheduled_tasks() -> impl Responder {
    let tasks = vec![
        ScheduledTask { id: 1, name: "Daily Backup".to_string(), description: "Perform daily backup of all databases".to_string(), schedule: "0 0 * * *".to_string(), target: "all".to_string(), status: "Scheduled".to_string() },
        ScheduledTask { id: 2, name: "Log Rotation".to_string(), description: "Rotate and compress log files".to_string(), schedule: "0 1 * * 0".to_string(), target: "nodes".to_string(), status: "Scheduled".to_string() },
        ScheduledTask { id: 3, name: "Security Scan".to_string(), description: "Run weekly security scan".to_string(), schedule: "0 2 * * 1".to_string(), target: "all".to_string(), status: "Scheduled".to_string() },
    ];
    web::Json(tasks)
}
