use actix_web::{get, web, Responder};
use serde::{Serialize, Deserialize};

#[derive(Serialize, Deserialize)]
struct Insight {
    title: String,
    description: String,
    link_text: String,
    cpu_change: f32,
    previous_release: String,
}

#[get("dashboard/insights")]
async fn get_insights() -> impl Responder {
    let insight = Insight {
        title: "Cluster average CPU usage is down since last release!".to_string(),
        description: "The average per-server CPU usage has decreased since the previous release.".to_string(),
        link_text: "See other stats since last release".to_string(),
        cpu_change: -1.2,
        previous_release: "1.2.1-A".to_string(),
    };

    web::Json(insight)
}