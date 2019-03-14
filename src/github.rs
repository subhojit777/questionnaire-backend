use crate::models;
use crate::AppState;
use actix_web::client;
use actix_web::client::ClientResponse;
use actix_web::error::Error;
use actix_web::http::Cookie;
use actix_web::AsyncResponder;
use actix_web::HttpMessage;
use actix_web::HttpRequest;
use actix_web::HttpResponse;
use dotenv::dotenv;
use error::Oauth as OauthError;
use futures::{Async, Future};
use std::collections::HashMap;
use std::env;
use std::str;

pub fn login_redirect(
    req: &HttpRequest<AppState>,
) -> Box<Future<Item = HttpResponse, Error = Error>> {
    dotenv().ok();

    let github_client_id = env::var("GITHUB_CLIENT_ID").expect("GITHUB_CLIENT_ID must be set.");
    let github_client_secret =
        env::var("GITHUB_CLIENT_SECRET").expect("GITHUB_CLIENT_SECRET must be set.");

    let mut query_string_map: HashMap<String, String> = HashMap::new();
    let query_strings: Vec<&str> = req.query_string().split('&').collect();

    for query_string in query_strings {
        if let Some(pos) = query_string.find('=') {
            let (parameter, value) = query_string.split_at(pos);

            query_string_map.insert(
                parameter.to_string(),
                value.trim_start_matches('=').to_string(),
            );
        }
    }

    let get_session_code: GetSessionCode = GetSessionCode {
        query_strings: query_string_map,
    };

    get_session_code
        .from_err()
        .and_then(|code| {
            let json_body = models::GHAccessTokenBody::new(
                github_client_id,
                github_client_secret,
                code,
                String::from("json"),
            );

            client::post("https://github.com/login/oauth/access_token")
                .json(json_body)
                .unwrap()
                .send()
                .from_err()
                .and_then(|res: ClientResponse| {
                    res.body().from_err().and_then(|body| {
                        let items: Vec<&str> =
                            str::from_utf8(body.as_ref()).unwrap().split('&').collect();
                        let mut token = Cookie::new("token", "");

                        for item in items {
                            let (key, value) = item.split_at(item.find('=').unwrap());

                            if key == "access_token" {
                                token.set_value(value.trim_matches('=').to_string());
                            }
                        }

                        Ok(HttpResponse::Ok().cookie(token).body(""))
                    })
                })
        })
        .responder()
}

/// Obtain session code from GitHub.
#[derive(Debug)]
struct GetSessionCode {
    query_strings: HashMap<String, String>,
}

impl Future for GetSessionCode {
    type Item = String;
    type Error = OauthError;

    fn poll(&mut self) -> Result<Async<Self::Item>, Self::Error> {
        let mut code = String::new();

        for (key, value) in self.query_strings.iter() {
            if key == "code" {
                code = value.to_string();
            }
        }

        if code == "" {
            return Err(OauthError::BadRequest);
        }

        Ok(Async::Ready(code))
    }
}
