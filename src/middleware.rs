use crate::error;
use actix_web::middleware::{Middleware, Started};
use actix_web::{Error, HttpRequest};
use helpers::header_map_wrapper::HeaderMapWrapper;

pub struct GitHubUser {
    id: i32,
}

impl<S> Middleware<S> for GitHubUser {
    fn start(&self, req: &HttpRequest<S>) -> Result<Started, Error> {
        let header_map: HeaderMapWrapper = HeaderMapWrapper {
            map: req.headers().clone(),
        };

        if let Some(token) = header_map.map.get("authorization") {
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
