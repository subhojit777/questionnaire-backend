use actix_web::ResponseError;
use actix_web::HttpResponse;
use crate::failure::Fail;
use actix_web::http::StatusCode;

/// Custom error handler for API requests.
#[derive(Fail, Debug)]
pub enum OauthError {
    #[fail(display = "bad request")]
    BadRequest,
}

impl ResponseError for OauthError {
    fn error_response(&self) -> HttpResponse {
        match *self {
            OauthError::BadRequest => HttpResponse::new(StatusCode::BAD_REQUEST),
        }
    }
}
