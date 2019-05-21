use actix_web::middleware::session::RequestSession;
use actix_web::{HttpRequest, HttpResponse};
use AppState;

pub fn logout(req: &HttpRequest<AppState>) -> HttpResponse {
    req.session().remove("gh_user_id");
    HttpResponse::Ok().finish()
}
