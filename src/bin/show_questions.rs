extern crate actix_web;
extern crate questionnaire_rs;
extern crate diesel;

use actix_web::{server, App, HttpRequest};
use questionnaire_rs::*;
use models::*;
use diesel::prelude::*;

//fn index(_req: &HttpRequest) -> &'static str {
//    "Hello world!"
//}

fn main() {
//    use self::schema::question::dsl::*;

    let connection = establish_connection();

    let results = schema::question::table.load::<Question>(&connection)
        .expect("Error loading questions");


    for question in results {
        println!("{}", question.title);
        println!("{}", question.created);
    }

//    server::new(|| App::new().resource("/", |r| r.f(index)))
//        .bind("127.0.0.1:8088")
//        .unwrap()
//        .run();
}
