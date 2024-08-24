use actix_web::{get, web, Responder};
use sqlx::sqlite::SqlitePool;
use sqlx::Row;


use crate::api::structs::*;


/// Handles GET requests for server information.
///
/// This endpoint retrieves server data from the database and returns it as JSON.
#[get("/servers")]
pub async fn get_servers(pool: web::Data<SqlitePool>) -> impl Responder {
    let query = "SELECT id, name, status, players, cpu, memory FROM servers";
    let rows = sqlx::query(query)
        .fetch_all(pool.get_ref())
        .await
        .unwrap_or_else(|_| vec![]); // Return an empty vector in case of error

    let servers: Vec<Server> = rows.into_iter().map(|row| Server {
        name: row.get("name"),
        status: row.get("status"),
        players: row.get("players"),
        cpu: row.get("cpu"),
        memory: row.get("memory"),
    }).collect();

    web::Json(servers)
}