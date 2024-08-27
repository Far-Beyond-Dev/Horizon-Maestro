use actix_web::{get, web, Responder};
use serde::Serialize;

#[derive(Serialize)]
struct ClusterUsageResponse {
    categories: Vec<String>,
    series: Vec<Series>,
}

#[derive(Serialize)]
struct Series {
    name: String,
    data: Vec<i32>,
    color: String,
}

#[get("/network/cluster-usage")]
async fn cluster_usage() -> impl Responder {
    let response = ClusterUsageResponse {
        categories: vec![
            "01 Feb", "02 Feb", "03 Feb", "04 Feb", "05 Feb", "06 Feb", "07 Feb"
        ].into_iter().map(String::from).collect(),
        series: vec![
            Series {
                name: "Cluster 1".to_string(),
                data: vec![75, 1, 70, 85, 90, 95, 88],
                color: "rgba(26, 86, 219, 1)".to_string(),
            },
            Series {
                name: "Cluster 2".to_string(),
                data: vec![65, 70, 75, 80, 85, 80, 82],
                color: "rgba(253, 186, 140, 1)".to_string(),
            },
            Series {
                name: "Cluster 3".to_string(),
                data: vec![55, 60, 65, 70, 75, 70, 78],
                color: "rgba(16, 185, 129, 1)".to_string(),
            },
            Series {
                name: "Cluster 4".to_string(),
                data: vec![45, 50, 55, 60, 65, 60, 68],
                color: "rgba(245, 158, 11, 1)".to_string(),
            },
        ],
    };

    web::Json(response)
}