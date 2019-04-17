use crate::error::Oauth;
use actix_web::middleware::{Middleware, Started};
use actix_web::{Error, HttpRequest};
use reqwest::header::AUTHORIZATION;
use reqwest::{Client, StatusCode};
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
                    // TODO: Only perform a request when user id is not present in session.
                    let client = Client::new();
                    let mut response = client
                        .get("https://api.github.com/user")
                        .header(AUTHORIZATION, access_token)
                        .send()
                        .expect("Unable to retrieve user id. Please check logs for details.");

                    if response.status() != StatusCode::OK {
                        return Err(Error::from(Oauth::BadRequest));
                    }

                    // TODO: Store user id in session.
                    dbg!(response.json::<GitHubResponse>().unwrap());

                    return Ok(Started::Done);
                }
                Err(_) => return Err(Error::from(Oauth::BadRequest)),
            };
        } else {
            Ok(Started::Done)
        }
    }
}
