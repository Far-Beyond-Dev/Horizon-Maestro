use sqlx::sqlite::SqlitePool;
use std::env;
use std::fs::File;

pub async fn setup_db() -> SqlitePool {
    let database_url = env::var("DATABASE_URL").unwrap_or_else(|_| "sqlite:mydb.db".to_string());

    
    // Attempt to create the file if it doesn't exist
    let file_creation_result = File::create("mydb.db");
    
    if let Err(e) = file_creation_result {
        eprintln!("Error creating database file: {}", e);
    }

    let pool = match SqlitePool::connect(&database_url).await {
        Ok(p) => p,
        Err(e) => {
            eprintln!("Failed to connect to the database: {}", e);
            panic!("Unable to establish a database connection");
        }
    };

    // Create the 'servers' table if it does not exist
    sqlx::query(
        "CREATE TABLE IF NOT EXISTS servers (
            id INTEGER PRIMARY KEY AUTOINCREMENT,
            name TEXT NOT NULL,
            status TEXT NOT NULL,
            players INTEGER NOT NULL,
            cpu REAL NOT NULL,
            memory REAL NOT NULL
        )"
    ).execute(&pool).await.unwrap();

    pool
}
