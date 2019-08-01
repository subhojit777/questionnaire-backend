use actix_web::middleware::session::RequestSession;
use actix_web::{HttpRequest, HttpResponse};
use AppState;

/// Logs out the user.
///
/// `/logout` GET
///
/// Headers:
///
/// ```txt
/// Authorization: token <access_token>
/// ```
///
/// Response: 200 OK
pub fn logout(req: &HttpRequest<AppState>) -> HttpResponse {
    req.session().clear();
    HttpResponse::Ok().finish()
}
