extern crate actix_web;
extern crate questionnaire_rs;
extern crate diesel;

use actix_web::{server, App, HttpRequest, HttpResponse, Json, http::Method};
use questionnaire_rs::*;
use models::*;
use diesel::prelude::*;
use std::fmt::Write;

fn index(_req: &HttpRequest) -> String {
    // TODO: Re-implement this. Check the original app for the home page content.
    use self::schema::question::dsl::*;

    // TODO: Connection should be established only once. Not per function.
    let connection = establish_connection();
    let mut output = String::new();

    let results = question.load::<Question>(&connection)
        .expect("Error loading questions");

    for row in results {
        write!(&mut output, "{}\n{}\n", row.title, row.created);
    }

    output
}

fn answer_post(answer: Json<AnswerForm>) -> HttpResponse {
    use schema::answers::dsl::*;

    // TODO: Connection should be established only once. Not per function.
    let connection = establish_connection();

    diesel::insert_into(answers)
        .values(&answer.into_inner())
        .execute(&connection)
        .expect("Error saving the answer");

    HttpResponse::Ok().finish()
}

fn main() {
    // TODO: URL should come from environment variable.
    server::new(|| App::new()
        .resource("/", |r| r.method(Method::GET).f(index))
        .resource("/answers", |r| r.method(Method::POST).with(answer_post)))
        .bind("127.0.0.1:8088")
        .unwrap()
        .run();
}
