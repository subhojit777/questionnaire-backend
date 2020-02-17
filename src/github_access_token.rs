use actix_web::client::Client;
use actix_web::client::ClientResponse;
use actix_web::{Error, HttpRequest, HttpResponse};
use dotenv::dotenv;
use futures::Future;
use models;
use std::collections::HashMap;
use std::{env, str};

/// `/gh-access-token` GET
///
/// Parameters:
///
/// code: GITHUB_LOGIN_CODE obtained from https://github.com/login/oauth/authorize
///
/// Response:
///
/// GITHUB_ACCESS_TOKEN in JSON.
pub fn get_access_token(req: HttpRequest) -> Box<dyn Future<Item = HttpResponse, Error = Error>> {
    dotenv().ok();

    // Creates a key-value pair of query strings in the request.
    let query_strings: HashMap<String, String> = req
        .query_string()
        .split('&')
        .map(|key_value| key_value.split('=').collect::<Vec<&str>>())
        .map(|vec| {
            assert_eq!(vec.len(), 2);
            (vec[0].to_string(), vec[1].to_string())
        })
        .collect();

    let code = query_strings.get("code").expect("Code not found.").clone();

    let github_client_id = env::var("GITHUB_CLIENT_ID").expect("GITHUB_CLIENT_ID must be set.");
    let github_client_secret =
        env::var("GITHUB_CLIENT_SECRET").expect("GITHUB_CLIENT_SECRET must be set.");

    let body = models::GHAccessTokenBody::new(
        github_client_id,
        github_client_secret,
        code,
        String::from("json"),
    );

    let mut client = Client::default();

    // In exchange of the code, retrieve the access token.
    // Throw error if it fails to retrieve the access token.
    client
        .post("https://github.com/login/oauth/access_token")
        .json(body)
        .unwrap()
        .send()
        .from_err()
        .and_then(|res: ClientResponse| {
            res.body().from_err().and_then(|body| {
                let items: Vec<&str> = str::from_utf8(body.as_ref()).unwrap().split('&').collect();
                let mut access_token = String::new();

                for item in items {
                    let (key, value) = item.split_at(item.find('=').unwrap());

                    if key == "access_token" {
                        access_token = value.trim_matches('=').to_string();
                    }
                }

                Ok(HttpResponse::Ok().json(access_token))
            })
        })
        .responder()
}
