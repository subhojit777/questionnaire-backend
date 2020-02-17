extern crate actix_web;
extern crate diesel;
extern crate questionnaire_rs;

use actix_web::HttpServer;
use questionnaire_rs::*;

fn main() {
    // TODO: URL should come from environment variable.
    HttpServer::new(create_app)
        .bind("127.0.0.1:8088")
        .unwrap()
        .run();
}
