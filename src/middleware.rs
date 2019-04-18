use crate::error::Oauth;
use actix_web::middleware::session::RequestSession;
use actix_web::middleware::{Middleware, Started};
use actix_web::{Error, HttpRequest};
use reqwest::header::AUTHORIZATION;
use reqwest::{Client, StatusCode};
use serde_derive::*;

pub struct GitHubUser;

#[derive(Deserialize, Serialize, Debug)]
pub struct GitHubResponse {
    id: i32,
}

impl<S> Middleware<S> for GitHubUser {
    fn start(&self, req: &HttpRequest<S>) -> Result<Started, Error> {
        if let Some(token) = req.headers().get("authorization") {
            match token.to_str() {
                Ok(access_token) => {
                    if let Some(gh_user_id) = req.session().get::<GitHubResponse>("gh_user_id")? {
                        return Ok(Started::Done);
                    } else {
                        let client = Client::new();
                        let mut response = client
                            .get("https://api.github.com/user")
                            .header(AUTHORIZATION, access_token)
                            .send()
                            .expect("Unable to retrieve user id. Please check logs for details.");

                        if response.status() != StatusCode::OK {
                            return Err(Error::from(Oauth::BadRequest));
                        }

                        req.session().set(
                            "gh_user_id",
                            response.json::<GitHubResponse>().expect(
                                "Unable to parse user id from response. Please check logs for details.",
                            ),
                        )?;
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
