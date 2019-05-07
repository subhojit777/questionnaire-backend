use crate::{error, AppState, DbExecutor};
use actix_web::{error as AWError, Path};

use actix_web::middleware::session::RequestSession;
use actix_web::{
    actix::{Handler, Message},
    AsyncResponder, HttpRequest, HttpResponse, Json, State,
};
use chrono::{DateTime, Utc};
use diesel::prelude::*;
use diesel::result::Error as DieselError;
use futures::future::IntoFuture;
use futures::Future;
use middleware::GitHubUserId;
use models::{Answer, AnswerInput, GetAnswerById, NewAnswer};

/// `/answers` POST
///
/// Headers:
///
/// Content type: application/json
/// Authorization: token <access_token>
///
/// Body:
/// ```json
/// {
///    "question_id": 23,
///    "title": "Nothing is as it seems."
/// }
/// ```
///
/// Response: 200 OK
pub fn post(
    data: Json<AnswerInput>,
    state: State<AppState>,
    req: HttpRequest<AppState>,
) -> Box<Future<Item = HttpResponse, Error = AWError::Error>> {
    let gh_user_id_session = req
        .session()
        .get::<GitHubUserId>("gh_user_id")
        .into_future();

    let now: DateTime<Utc> = Utc::now();

    gh_user_id_session
        .from_err()
        .and_then(move |gh_user_id| {
            let input = data.into_inner();
            let new_answer = NewAnswer::new(
                input.question_id,
                input.title,
                gh_user_id.unwrap().id,
                now.naive_utc(),
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

/// `/answers/{id}` GET
///
/// Headers:
///
/// Authorization: token <access_token>
///
/// Response:
/// ```json
/// {
///    "id": 47,
///    "question_id": 23,
///    "title": "Nothing is as it seems.",
///    "user_id": 7,
///    "created": "2019-11-01T14:30:30"
/// }
/// ```
pub fn get(
    data: Path<GetAnswerById>,
    req: HttpRequest<AppState>,
) -> Box<Future<Item = HttpResponse, Error = AWError::Error>> {
    req.state()
        .db
        .send(data.into_inner())
        .from_err()
        .and_then(|response| match response {
            Ok(result) => Ok(HttpResponse::Ok().json(result)),
            Err(DieselError::NotFound) => Ok(HttpResponse::NotFound().into()),
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

impl Message for GetAnswerById {
    type Result = Result<Answer, DieselError>;
}

impl Handler<GetAnswerById> for DbExecutor {
    type Result = Result<Answer, DieselError>;

    fn handle(&mut self, msg: GetAnswerById, _ctx: &mut Self::Context) -> Self::Result {
        use schema::answers::dsl::{answers, id};

        let connection: &MysqlConnection = &self.0.get().unwrap();

        let result: Answer = answers.filter(id.eq(&msg.0)).first(connection)?;

        Ok(result)
    }
}
