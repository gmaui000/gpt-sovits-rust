use actix_web::{web, HttpRequest, HttpResponse};
use std::sync::{Arc, Mutex};
use super::super::super::AppState;

#[actix_web::get("/")]
pub async fn index(data: web::Data<Arc<Mutex<AppState>>>, _req: HttpRequest) -> HttpResponse {
    let app_state = data.lock().unwrap();

    HttpResponse::Ok()
    .content_type("text/plain; charset=utf-8")
    .body(app_state.track.to_table_string())
}