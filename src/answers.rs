use crate::{error, AppState, DbExecutor};
use actix::{Handler, Message};
use actix_web::{error as AWError, web::Path};
use actix_web::{web::Json, HttpRequest, HttpResponse};
use chrono::{DateTime, Utc};
use diesel::prelude::*;
use diesel::result::Error as DieselError;
use futures::future::IntoFuture;
use futures::Future;
use middleware::GitHubUserId;
use models::{Answer, AnswerInput, GetAnswerById, NewAnswer};
use GH_USER_SESSION_ID_KEY;

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
///    "option_id": 23
/// }
/// ```
///
/// Response: 200 OK
pub fn post(
    data: Json<AnswerInput>,
    req: HttpRequest<AppState>,
) -> Box<Future<Item = HttpResponse, Error = AWError::Error>> {
    let state: &AppState = req.state();

    let gh_user_id_session = req
        .session()
        .get::<GitHubUserId>(GH_USER_SESSION_ID_KEY)
        .into_future();

    let now: DateTime<Utc> = Utc::now();

    gh_user_id_session
        .from_err()
        .and_then(move |gh_user_id| {
            let input = data.into_inner();
            let new_answer =
                NewAnswer::new(gh_user_id.unwrap().id, now.naive_utc(), input.option_id);

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
///    "user_id": 7,
///    "created": "2019-11-01T14:30:30",
///    "option_id": 23
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
