use crate::{AppState, DbExecutor};
use actix_web::{
    actix::{Handler, Message},
    error::Error,
    AsyncResponder, HttpRequest, HttpResponse, Json, State,
};
use diesel::prelude::*;
use futures::Future;
use models::{Answer, AnswerForm};

pub fn post(
    answer_form: Json<AnswerForm>,
    state: State<AppState>,
    _req: HttpRequest<AppState>,
) -> Box<Future<Item = HttpResponse, Error = Error>> {
    let answer = answer_form.into_inner();

    state
        .db
        .send(answer)
        .from_err()
        .and_then(|response| match response {
            Ok(result) => Ok(HttpResponse::Ok().json(result)),
            Err(_) => Ok(HttpResponse::InternalServerError().into()),
        })
        .responder()
}

impl Message for AnswerForm {
    type Result = Result<Answer, Error>;
}

impl Handler<AnswerForm> for DbExecutor {
    type Result = Result<Answer, Error>;

    fn handle(&mut self, msg: AnswerForm, _: &mut Self::Context) -> Self::Result {
        use schema::answers::dsl::{answers, question_id, user_id};

        let connection: &MysqlConnection = &self.0.get().unwrap();

        diesel::insert_into(answers)
            .values(&msg)
            .execute(connection)
            .expect("Error saving the answer_form");

        let result: Answer = answers
            .filter(question_id.eq(&msg.question_id))
            .filter(user_id.eq(&msg.user_id))
            .first(connection)
            .unwrap();

        Ok(result)
    }
}
