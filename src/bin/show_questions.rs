extern crate actix_web;
extern crate questionnaire_rs;
extern crate diesel;

use actix_web::{server, App, HttpRequest, HttpResponse, Json, http::Method};
use questionnaire_rs::*;
use models::*;
use diesel::prelude::*;
use std::fmt::Write;

fn index(_req: &HttpRequest) -> String {
    use self::schema::question::dsl::*;

    let connection = establish_connection();
    let mut output = String::new();

    let results = question.load::<Question>(&connection)
        .expect("Error loading questions");

    for row in results {
        write!(&mut output, "{}\n{}\n", row.title, row.created);
    }

    output
}

fn submit_answer(answer: Json<Answer>) -> HttpResponse {
    println!("{:?}", answer);
    HttpResponse::Ok().finish()
}

fn main() {
    server::new(|| App::new()
        .resource("/", |r| r.method(Method::GET).f(index))
        .resource("/submit-answer", |r| r.method(Method::POST).with(submit_answer)))
        .bind("127.0.0.1:8088")
        .unwrap()
        .run();
}
