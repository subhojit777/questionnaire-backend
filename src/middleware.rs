use crate::error::Oauth;
use actix_web::client::ClientResponse;
use actix_web::http::StatusCode;
use actix_web::middleware::{Middleware, Started};
use actix_web::{client, Error, HttpRequest, HttpResponse};
use futures::future;
use futures::future::Future;

pub struct GitHubUser;

impl<S> Middleware<S> for GitHubUser {
    fn start(&self, req: &HttpRequest<S>) -> Result<Started, Error> {
        if let Some(token) = req.headers().get("authorization") {
            match token.to_str() {
                Ok(access_token) => {
                    let gh_user_future = client::get("https://api.github.com/user")
                        .header("Authorization", access_token)
                        .finish()
                        .unwrap()
                        .send()
                        .from_err()
                        .and_then(|res: ClientResponse| match res.status() {
                            StatusCode::OK => {
                                return future::ok(None);
                            }
                            _ => return future::ok(Some(HttpResponse::BadRequest().finish())),
                        });

                    return Ok(Started::Future(Box::new(gh_user_future)));
                }
                Err(_) => return Err(Error::from(Oauth::BadRequest)),
            };
        } else {
            Ok(Started::Done)
        }
    }
}
