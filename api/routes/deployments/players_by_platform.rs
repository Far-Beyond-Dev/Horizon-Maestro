use actix_web::{get, web, Responder};
use serde::{Serialize, Deserialize};

#[derive(Serialize, Deserialize)]
struct DataPoint {
    x: String,
    y: i32,
}

#[derive(Serialize, Deserialize)]
struct Series {
    name: String,
    color: String,
    data: Vec<DataPoint>,
}

#[get("/deployments/players-by-platform")]
async fn players_by_platform() -> impl Responder {
    let data = vec![
        Series {
            name: "Desktop PC".to_string(),
            color: "#1A56DB".to_string(),
            data: vec![
                DataPoint { x: "01 Feb".to_string(), y: 170 },
                DataPoint { x: "02 Feb".to_string(), y: 180 },
                DataPoint { x: "03 Feb".to_string(), y: 164 },
                DataPoint { x: "04 Feb".to_string(), y: 145 },
                DataPoint { x: "05 Feb".to_string(), y: 194 },
                DataPoint { x: "06 Feb".to_string(), y: 170 },
                DataPoint { x: "07 Feb".to_string(), y: 155 },
            ],
        },
        Series {
            name: "Phones".to_string(),
            color: "##ff00ff".to_string(),
            data: vec![
                DataPoint { x: "01 Feb".to_string(), y: 32 },
                DataPoint { x: "02 Feb".to_string(), y: 32 },
                DataPoint { x: "03 Feb".to_string(), y: 32 },
                DataPoint { x: "04 Feb".to_string(), y: 32 },
                DataPoint { x: "05 Feb".to_string(), y: 32 },
                DataPoint { x: "06 Feb".to_string(), y: 32 },
                DataPoint { x: "07 Feb".to_string(), y: 32 },
            ],
        },
        Series {
            name: "Gaming/Console".to_string(),
            color: "#17B0BD".to_string(),
            data: vec![
                DataPoint { x: "01 Feb".to_string(), y: 2 },
                DataPoint { x: "02 Feb".to_string(), y: 1 },
                DataPoint { x: "03 Feb".to_string(), y: 2 },
                DataPoint { x: "04 Feb".to_string(), y: 2 },
                DataPoint { x: "05 Feb".to_string(), y: 2 },
                DataPoint { x: "06 Feb".to_string(), y: 2 },
                DataPoint { x: "07 Feb".to_string(), y: 1 },
            ],
        },
    ];
    web::Json(data)
}