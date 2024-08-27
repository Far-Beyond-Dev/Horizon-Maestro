use actix_web::{get, web, HttpResponse, Responder};
use sqlx::sqlite::SqlitePool;
use sqlx::Row;
use serde::Deserialize;
use crate::api::structs::*;

#[derive(Deserialize)]
struct PageParams {
    page: Option<u32>,
}

#[derive(serde::Serialize)]
struct PaginatedResponse {
    servers: Vec<Server>,
    total: i64,
}

/// Handles GET requests for server information.
///
/// This endpoint retrieves paginated server data from the database and returns it as JSON.
#[get("/servers")]
pub async fn get_servers(
    pool: web::Data<SqlitePool>,
    web::Query(params): web::Query<PageParams>
) -> impl Responder {
    let page = params.page.unwrap_or(1);
    let per_page = 20;
    let offset = (page - 1) * per_page;

    // Query to get paginated servers
    let query = "SELECT id, name, status, players, cpu, memory FROM servers LIMIT ? OFFSET ?";
    let rows = sqlx::query(query)
        .bind(per_page)
        .bind(offset)
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

    // Query to get total count of servers
    let count_query = "SELECT COUNT(*) as count FROM servers";
    let total: i64 = sqlx::query(count_query)
        .fetch_one(pool.get_ref())
        .await
        .map(|row: sqlx::sqlite::SqliteRow| row.get("count"))
        .unwrap_or(0);

    let response = PaginatedResponse {
        servers,
        total,
    };

    HttpResponse::Ok().json(response)
}