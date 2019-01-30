use crate::AppState;
use crate::models;
use actix_web::actix;
use actix_web::client;
use actix_web::client::ClientResponse;
use actix_web::error::Error;
use actix_web::AsyncResponder;
use actix_web::FutureResponse;
use actix_web::HttpRequest;
use actix_web::HttpResponse;
use dotenv::dotenv;
use futures::Future;
use serde_derive::Deserialize;
use serde_derive::Serialize;
use std::env;

pub fn login_page(_: &HttpRequest<AppState>) -> HttpResponse {
    dotenv().ok();
    let github_client_id = env::var("GITHUB_CLIENT_ID").expect("GITHUB_CLIENT_ID must be set.");

    let body = format!("<html><body><a href=\"https://github.com/login/oauth/authorize?scope=user:email&client_id={}\">Click here to login</a></body></html>", github_client_id);

    HttpResponse::Ok().content_type("text/html").body(body)
}

pub fn login_redirect(
    req: &HttpRequest<AppState>,
) -> Box<Future<Item = HttpResponse, Error = Error>> {
    let query_strings = req.query_string().split('&');
    let mut session_code = String::new();

    for query_string in query_strings {
        let (parameter, value) = query_string.split_at(query_string.find('=').unwrap());

        if parameter == "code" {
            session_code = value.trim_start_matches('=').to_string();
        }
    }

    dotenv().ok();

    let github_client_id = env::var("GITHUB_CLIENT_ID").expect("GITHUB_CLIENT_ID must be set.");
    let github_client_secret =
        env::var("GITHUB_CLIENT_SECRET").expect("GITHUB_CLIENT_SECRET must be set.");

    let json_body = models::GHAccessTokenBody::new(
        github_client_id,
        github_client_secret,
        session_code,
        String::from("json"),
    );

    client::post("https://github.com/login/oauth/access_token")
        .json(json_body)
        .unwrap()
        .send()
        .from_err()
        .and_then(|_res: ClientResponse| {
            Ok(HttpResponse::Ok().body("inside future"))
        })
        .responder()
}
