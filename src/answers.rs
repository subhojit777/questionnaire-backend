extern crate actix_web;
extern crate diesel;

use models::AnswerForm;
use self::actix_web::{Json, HttpResponse};
use diesel::prelude::*;

pub fn post(answer: Json<AnswerForm>) -> HttpResponse {
    use schema::answers::dsl::*;

    // TODO: Connection should be established only once. Not per function.
    let connection = ::establish_connection();

    diesel::insert_into(answers)
        .values(&answer.into_inner())
        .execute(&connection)
        .expect("Error saving the answer");

    HttpResponse::Ok().finish()
}
