use crate::failure::Fail;
use actix_web::http::StatusCode;
use actix_web::{HttpResponse, ResponseError};

// Custom error handler for failed db transactions.
#[derive(Fail, Debug)]
pub enum Db {
    #[fail(display = "failed transaction")]
    FailedTransaction,
}

impl ResponseError for Db {
    fn error_response(&self) -> HttpResponse {
        match *self {
            Db::FailedTransaction => HttpResponse::new(StatusCode::INTERNAL_SERVER_ERROR),
        }
    }
}
