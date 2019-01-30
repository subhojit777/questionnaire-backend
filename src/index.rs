use crate::AppState;
use actix_web::HttpRequest;
use actix_web::HttpResponse;

pub fn get(_req: &HttpRequest<AppState>) -> HttpResponse {
    HttpResponse::Ok()
        .content_type("text/html")
        .body("<html><body><a href=\"\"></a></body></html>")
}
