use actix_session::Session;
use actix_web::{get, HttpRequest, HttpResponse};

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
#[get("/logout")]
pub fn logout(session: Session) -> HttpResponse {
    session.clear();
    HttpResponse::Ok().finish()
}
