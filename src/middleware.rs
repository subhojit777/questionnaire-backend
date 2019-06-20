use crate::error::Oauth;
use actix_web::middleware::session::RequestSession;
use actix_web::middleware::{Middleware, Started};
use actix_web::{Error, HttpRequest};
use reqwest::header::AUTHORIZATION;
use reqwest::{Client, StatusCode};
use serde_derive::*;
use GH_USER_SESSION_ID_KEY;

/// Sets the GitHub user id in request - if not already present.
#[derive(Deserialize, Serialize, Debug)]
pub struct GitHubUserId {
    pub id: i32,
}

impl GitHubUserId {
    pub fn default() -> Self {
        GitHubUserId { id: -1 }
    }
}

impl<S> Middleware<S> for GitHubUserId {
    fn start(&self, req: &HttpRequest<S>) -> Result<Started, Error> {
        if let Some(token) = req.headers().get("authorization") {
            match token.to_str() {
                Ok(access_token) => {
                    if let Some(_) = req.session().get::<GitHubUserId>(GH_USER_SESSION_ID_KEY)? {
                    } else {
                        // Using synchronous reqwest to retrieve user_id from GitHub.
                        // Actix doesn't yet provides a synchronous API. And doing it using futures
                        // felt unnecessary. Actix even uses futures to parse a JSON response which
                        // I felt is completely unnecessary here.
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
                            GH_USER_SESSION_ID_KEY,
                            response.json::<GitHubUserId>().expect(
                                "Unable to parse user id from response. Please check logs for details.",
                            ),
                        )?;
                    }

                    return Ok(Started::Done);
                }
                Err(_) => return Err(Error::from(Oauth::BadRequest)),
            };
        } else {
            Err(Error::from(Oauth::BadRequest))
        }
    }
}
