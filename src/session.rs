use actix_web::{HttpRequest, HttpResponse};

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
pub fn logout(req: &HttpRequest) -> HttpResponse {
    req.session().clear();
    HttpResponse::Ok().finish()
}
