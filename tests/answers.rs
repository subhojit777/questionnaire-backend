extern crate actix_web;
extern crate questionnaire_rs;
extern crate serde_json;

use actix_web::{http::Method, test::TestServer, HttpMessage};
use questionnaire_rs::models::{Answer, AnswerInput};
use std::str;

#[test]
fn post() {
    let mut server = TestServer::with_factory(questionnaire_rs::create_app);

    let answer_form = AnswerInput {
        question_id: 10,
        title: String::from("Some answer"),
        user_id: 1,
    };
    let request = server
        .client(Method::POST, "/answers")
        .json(answer_form)
        .unwrap();
    let response = server.execute(request.send()).unwrap();

    let bytes = server.execute(response.body()).unwrap();
    let body = str::from_utf8(&bytes).unwrap();
    let answer = serde_json::from_str::<Answer>(body).unwrap();

    assert!(response.status().is_success());
    assert_eq!(answer.question_id, 10);
    assert_eq!(answer.title, String::from("Some answer"));
    assert_eq!(answer.user_id, 1);
}
