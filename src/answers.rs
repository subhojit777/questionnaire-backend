use crate::{error, AppState, DbExecutor};
use actix_web::{error as AWError, Path};

use actix_web::{
    actix::{Handler, Message},
    AsyncResponder, HttpRequest, HttpResponse, Json, State,
};
use diesel::prelude::*;
use futures::Future;
use models::{Answer, AnswerForm, AnswerId};

pub fn post(
    answer_form: Json<AnswerForm>,
    state: State<AppState>,
    _req: HttpRequest<AppState>,
) -> Box<Future<Item = HttpResponse, Error = AWError::Error>> {
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

/// TODO: This is not yet fully implemented. It is supposed to return an answer based on an ID.
/// The code inside explains how the oauth wrapper is supposed to work.
pub fn get(
    data: Path<AnswerId>,
    req: HttpRequest<AppState>,
) -> Box<Future<Item = HttpResponse, Error = AWError::Error>> {
    req.state()
        .db
        .send(data.into_inner())
        .from_err()
        .and_then(|response| match response {
            Ok(result) => Ok(HttpResponse::Ok().json(result)),
            Err(_) => Ok(HttpResponse::InternalServerError().into()),
        })
        .responder()
}

impl Message for AnswerForm {
    type Result = Result<Answer, error::Db>;
}

impl Handler<AnswerForm> for DbExecutor {
    type Result = Result<Answer, error::Db>;

    fn handle(&mut self, msg: AnswerForm, _: &mut Self::Context) -> Self::Result {
        use schema::answers::dsl::{answers, question_id, user_id};

        let connection: &MysqlConnection = &self.0.get().unwrap();

        diesel::insert_into(answers)
            .values(&msg)
            .execute(connection)
            .expect("Error saving the answer_form");

        let result: QueryResult<Answer> = answers
            .filter(
                question_id
                    .eq(&msg.question_id)
                    .and(user_id.eq(&msg.user_id)),
            )
            .first(connection);

        match result {
            Ok(answer) => Ok(answer),
            Err(_) => Err(error::Db),
        }
    }
}

impl Message for AnswerId {
    type Result = Result<Answer, error::Db>;
}

impl Handler<AnswerId> for DbExecutor {
    type Result = Result<Answer, error::Db>;

    fn handle(&mut self, msg: AnswerId, _ctx: &mut Self::Context) -> Self::Result {
        use schema::answers::dsl::{answers, id};

        let connection: &MysqlConnection = &self.0.get().unwrap();

        let result: QueryResult<Answer> = answers.filter(id.eq(&msg.0)).first(connection);

        match result {
            Ok(answer) => Ok(answer),
            Err(_) => Err(error::Db),
        }
    }
}
