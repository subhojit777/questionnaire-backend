extern crate actix_web;
extern crate diesel;
extern crate questionnaire_rs;
extern crate dotenv;

use actix_web::server;
use questionnaire_rs::*;
use dotenv::dotenv;
use std::env;

fn main() {
    dotenv().ok();
    let client_url = env::var("CLIENT_URL").expect("CLIENT_URL (example: 127.0.0.1:8088) must be set.");

    server::new(create_app)
        .bind(client_url)
        .unwrap()
        .run();
}
