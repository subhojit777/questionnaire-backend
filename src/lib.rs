extern crate chrono;
#[macro_use]
extern crate diesel;
extern crate actix_web;
extern crate dotenv;
extern crate serde_derive;

use actix_web::{http::Method, App};
use diesel::mysql::MysqlConnection;
use diesel::prelude::*;
use dotenv::dotenv;
use std::env;

pub mod answers;
pub mod index;
pub mod models;
pub mod schema;

pub fn establish_connection() -> MysqlConnection {
    dotenv().ok();

    let database_url = env::var("DATABASE_URL").expect("DATABASE_URL must be set.");
    MysqlConnection::establish(&database_url)
        .expect(&format!("Error connecting to {}", database_url))
}

pub fn create_app() -> App {
    App::new()
        .resource("/", |r| r.method(Method::GET).f(index::get))
        .resource("/answers", |r| r.method(Method::POST).with(answers::post))
}
