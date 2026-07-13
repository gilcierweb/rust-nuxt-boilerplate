use actix_web::{HttpResponse, HttpRequest};

pub async fn metrics(_req: HttpRequest) -> HttpResponse {
    HttpResponse::Ok()
        .content_type("text/plain; version=0.0.4")
        .body("# No metrics collected yet\n")
}
