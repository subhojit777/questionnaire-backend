use crate::{error, AppState, DbExecutor};
use actix_web::{error as AWError, Path};

use actix_web::middleware::session::RequestSession;
use actix_web::{
    actix::{Handler, Message},
    AsyncResponder, HttpRequest, HttpResponse, Json, State,
};
use diesel::prelude::*;
use futures::future::IntoFuture;
use futures::Future;
use middleware::GitHubResponse;
use models::{Answer, AnswerId, AnswerInput, NewAnswer};

pub fn post(
    data: Json<AnswerInput>,
    state: State<AppState>,
    req: HttpRequest<AppState>,
) -> Box<Future<Item = HttpResponse, Error = AWError::Error>> {
    let gh_user_id_session = req
        .session()
        .get::<GitHubResponse>("gh_user_id")
        .into_future();

    gh_user_id_session
        .from_err()
        .and_then(move |gh_user_id| {
            let input = data.into_inner();
            let new_answer = NewAnswer::new(
                input.question_id,
                input.title,
                gh_user_id.unwrap().id,
                input.created,
            );

            state
                .db
                .send(new_answer)
                .from_err()
                .and_then(|response| match response {
                    Ok(_) => Ok(HttpResponse::Ok().finish()),
                    Err(_) => Ok(HttpResponse::InternalServerError().into()),
                })
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

impl Message for NewAnswer {
    type Result = Result<(), error::Db>;
}

impl Handler<NewAnswer> for DbExecutor {
    type Result = Result<(), error::Db>;

    fn handle(&mut self, msg: NewAnswer, _: &mut Self::Context) -> Self::Result {
        use schema::answers::dsl::answers;

        let connection: &MysqlConnection = &self.0.get().unwrap();

        diesel::insert_into(answers)
            .values(&msg)
            .execute(connection)
            .expect("Error saving the an answer");

        Ok(())
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
