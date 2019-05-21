use actix_web::middleware::session::RequestSession;
use actix_web::{HttpRequest, HttpResponse};
use {AppState, GH_USER_SESSION_ID_KEY};

pub fn logout(req: &HttpRequest<AppState>) -> HttpResponse {
    req.session().remove(GH_USER_SESSION_ID_KEY);
    HttpResponse::Ok().finish()
}
