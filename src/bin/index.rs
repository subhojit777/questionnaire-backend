extern crate actix_web;
extern crate diesel;
extern crate dotenv;
extern crate questionnaire_rs;

use actix_web::server;
use dotenv::dotenv;
use questionnaire_rs::*;
use std::env;

fn main() {
    dotenv().ok();
    let client_url =
        env::var("CLIENT_URL").expect("CLIENT_URL (example: 127.0.0.1:8088) must be set.");

    server::new(create_app).bind(client_url).unwrap().run();
}
