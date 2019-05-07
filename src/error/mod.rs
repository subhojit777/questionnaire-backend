use crate::failure::Fail;
use actix_web::http::StatusCode;
use actix_web::{error::ResponseError, HttpResponse};
use std::fmt::{Display, Formatter, Result};

/// Custom error handler for failed db transactions.
#[derive(Fail, Debug)]
pub struct Db;

impl ResponseError for Db {
    fn error_response(&self) -> HttpResponse {
        HttpResponse::new(StatusCode::INTERNAL_SERVER_ERROR)
    }
}

impl Display for Db {
    fn fmt(&self, f: &mut Formatter) -> Result {
        write!(f, "{:?}", self)
    }
}

/// Custom error handler for API requests.
#[derive(Fail, Debug)]
pub enum Oauth {
    #[fail(display = "bad request")]
    BadRequest,
}

impl ResponseError for Oauth {
    fn error_response(&self) -> HttpResponse {
        match *self {
            Oauth::BadRequest => HttpResponse::new(StatusCode::BAD_REQUEST),
        }
    }
}
