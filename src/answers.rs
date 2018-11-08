extern crate actix_web;
extern crate diesel;
extern crate serde_json;

use self::actix_web::{HttpResponse, Json};
use diesel::prelude::*;
use models::{Answer, AnswerForm};

pub fn post(answer_form: Json<AnswerForm>) -> HttpResponse {
    use schema::answers::dsl::{answers, question_id, title, user_id};

    // TODO: Connection should be established only once. Not per function.
    let connection = ::establish_connection();
    let answer = answer_form.into_inner();

    diesel::insert_into(answers)
        .values(&answer)
        .execute(&connection)
        .expect("Error saving the answer_form");

    let result: Answer = answers
        .filter(question_id.eq(&answer.question_id))
        .filter(title.eq(&answer.title))
        .filter(user_id.eq(&answer.user_id))
        .first(&connection)
        .unwrap();

    HttpResponse::Ok().json(result)
}
