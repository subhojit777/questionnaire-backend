use crate::error;
use actix_web::http::HeaderMap;
use futures::{Async, Future};

/// Future implementation of actix_web::http::HeaderMap.
pub struct HeaderMapWrapper {
    pub map: HeaderMap,
}

impl Future for HeaderMapWrapper {
    type Item = String;
    type Error = error::Oauth;

    fn poll(&mut self) -> Result<Async<Self::Item>, Self::Error> {
        if let Some(token) = self.map.get("authorization") {
            match token.to_str() {
                Ok(val) => return Ok(Async::Ready(val.to_string())),
                Err(_) => return Err(error::Oauth::BadRequest),
            };
        } else {
            Err(error::Oauth::BadRequest)
        }
    }
}
