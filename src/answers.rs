use crate::{DbPool, GH_USER_SESSION_ID_KEY};
use actix_web::Error;

use crate::middleware::GitHubUserId;
use crate::models::{Answer, AnswerInput, GetAnswerById, GetAnswersByOption, NewAnswer};
use actix_session::Session;
use actix_web::web::{block, Data, Json, Path, Query};
use actix_web::{get, post};
use actix_web::{HttpRequest, HttpResponse};
use chrono::{DateTime, Utc};
use diesel::prelude::*;
use diesel::result::Error as DieselError;
use futures::Future;
use serde_json::ser::State;

pub fn new_answer(
    option_id: i32,
    user_id: i32,
    connection: &MysqlConnection,
) -> Result<(), DieselError> {
    use crate::schema::answers::dsl::answers;

    let new_answer = NewAnswer::new(user_id, Utc::now().naive_utc(), option_id);

    diesel::insert_into(answers)
        .values(&new_answer)
        .execute(connection)
        .expect("Error saving an answer");

    Ok(())
}

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
#[post("/answers")]
pub async fn post(
    pool: Data<DbPool>,
    session: Session,
    data: Json<AnswerInput>,
) -> Result<HttpResponse, Error> {
    let gh_user_id_session = session
        .get::<GitHubUserId>(GH_USER_SESSION_ID_KEY)
        .unwrap_or_else(|_| Some(GitHubUserId { id: 1 }));

    return if let Some(user_id) = gh_user_id_session {
        let connection = pool.get().expect("couldn't get db connection from pool");
        let input = data.into_inner();

        block(move || new_answer(input.option_id, user_id.id, &connection))
            .await
            .map_err(|_| HttpResponse::InternalServerError().finish())?;

        Ok(HttpResponse::Ok().finish())
    } else {
        Ok(HttpResponse::BadRequest().finish())
    };
}

/// `/answers/{id}` GET
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
pub async fn get(data: Path<GetAnswerById>, req: HttpRequest) -> Result<HttpResponse, Error> {
    Ok(HttpResponse::Ok().finish())
    // req.state()
    //     .db
    //     .send(data.into_inner())
    //     .from_err()
    //     .and_then(|response| match response {
    //         Ok(result) => Ok(HttpResponse::Ok().json(result)),
    //         Err(DieselError::NotFound) => Ok(HttpResponse::NotFound().into()),
    //         Err(_) => Ok(HttpResponse::InternalServerError().into()),
    //     })
    //     .responder()
}

/// Returns answers for an option.
///
/// `/answers-option` GET
///
/// Parameters:
///
/// option_id: {id}
///
/// Response:
/// ```json
/// [
///    {
///         "id": 12,
///         "user_id": 9,
///         "created": "2019-06-19T03:40:50",
///         "option_id": 1,
///     },
///    {
///         "id": 13,
///         "user_id": 18,
///         "created": "2019-06-30T03:40:50",
///         "option_id": 3,
///     }
/// ]
/// ```
pub async fn get_by_option(
    data: Query<GetAnswersByOption>,
    req: HttpRequest,
) -> Result<HttpResponse, Error> {
    Ok(HttpResponse::Ok().finish())
    // let state = req.state();
    //
    // state
    //     .db
    //     .send(data.into_inner())
    //     .from_err()
    //     .and_then(|response| match response {
    //         Ok(result) => Ok(HttpResponse::Ok().json(result)),
    //         Err(DieselError::NotFound) => Ok(HttpResponse::NotFound().into()),
    //         Err(_) => Ok(HttpResponse::InternalServerError().into()),
    //     })
    //     .responder()
}

// impl Message for NewAnswer {
//     type Result = Result<(), error::Db>;
// }
//
// impl Handler<NewAnswer> for DbExecutor {
//     type Result = Result<(), error::Db>;
//
//     fn handle(&mut self, msg: NewAnswer, _: &mut Self::Context) -> Self::Result {
//         use crate::schema::answers::dsl::answers;
//
//         let connection: &MysqlConnection = &self.0.get().unwrap();
//
//         diesel::insert_into(answers)
//             .values(&msg)
//             .execute(connection)
//             .expect("Error saving the an answer");
//
//         Ok(())
//     }
// }

// impl Message for GetAnswerById {
//     type Result = Result<Answer, DieselError>;
// }
//
// impl Handler<GetAnswerById> for DbExecutor {
//     type Result = Result<Answer, DieselError>;
//
//     fn handle(&mut self, msg: GetAnswerById, _ctx: &mut Self::Context) -> Self::Result {
//         use crate::schema::answers::dsl::{answers, id};
//
//         let connection: &MysqlConnection = &self.0.get().unwrap();
//
//         let result: Answer = answers.filter(id.eq(&msg.0)).first(connection)?;
//
//         Ok(result)
//     }
// }
//
// impl Message for GetAnswersByOption {
//     type Result = Result<Vec<Answer>, DieselError>;
// }
//
// impl Handler<GetAnswersByOption> for DbExecutor {
//     type Result = Result<Vec<Answer>, DieselError>;
//
//     fn handle(&mut self, msg: GetAnswersByOption, _ctx: &mut Self::Context) -> Self::Result {
//         use crate::schema::answers;
//         use crate::schema::answers::dsl::option_id;
//
//         let connection: &MysqlConnection =
//             &self.0.get().expect("Unable to get database connection.");
//
//         let answers: Vec<Answer> = answers::table
//             .filter(option_id.eq(msg.option_id))
//             .load(connection)?;
//
//         Ok(answers)
//     }
// }
