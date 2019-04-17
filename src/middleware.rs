use crate::error::Oauth;
use actix_web::middleware::{Middleware, Started};
use actix_web::{Error, HttpRequest};
use reqwest::header::AUTHORIZATION;
use reqwest::Client;
use serde_derive::*;

pub struct GitHubUser;

#[derive(Deserialize, Debug)]
pub struct GitHubResponse {
    id: i32,
}

impl<S> Middleware<S> for GitHubUser {
    fn start(&self, req: &HttpRequest<S>) -> Result<Started, Error> {
        if let Some(token) = req.headers().get("authorization") {
            match token.to_str() {
                Ok(access_token) => {
                    let client = Client::new();
                    let mut response = client
                        .get("https://api.github.com/user")
                        .header(AUTHORIZATION, access_token)
                        .send();
                    match response {
                        Ok(mut response_2) => {
                            dbg!(response_2.json::<GitHubResponse>().unwrap());
                        }
                        Err(_) => {}
                    }
                    return Ok(Started::Done);
                }
                Err(_) => return Err(Error::from(Oauth::BadRequest)),
            };
        } else {
            Ok(Started::Done)
        }
    }
}
