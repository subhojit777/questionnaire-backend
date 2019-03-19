use crate::error;
use actix_web::middleware::{Middleware, Started};
use actix_web::{Error, HttpRequest};

pub struct GitHubUser {
    id: i32,
}

impl<S> Middleware<S> for GitHubUser {
    fn start(&self, req: &HttpRequest<S>) -> Result<Started, Error> {
        if let Some(token) = req.headers().get("authorization") {
            match token.to_str() {
                Ok(_val) => {
                    // TODO: Do GET https://api.github.com/user here to retrieve user id.
                    return Ok(Started::Done);
                }
                Err(_) => return Err(Error::from(error::Oauth::BadRequest)),
            };
        } else {
            Ok(Started::Done)
        }
    }
}
