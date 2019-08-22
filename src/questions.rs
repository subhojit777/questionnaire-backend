use actix::{Handler, Message};
use actix_web::middleware::session::RequestSession;
use actix_web::{AsyncResponder, Path, Query};
use actix_web::{Error, HttpRequest, HttpResponse, Json, State};
use chrono::Utc;
use diesel::prelude::*;
use diesel::result::Error as DieselError;
use futures::Future;
use futures::IntoFuture;
use middleware::GitHubUserId;
use models::{GetQuestion, GetQuestionByPresentation, NewQuestion, NewQuestionJson, Questions};
use GH_USER_SESSION_ID_KEY;
use {AppState, DbExecutor};

impl Message for NewQuestion {
    type Result = Result<(), DieselError>;
}

impl Handler<NewQuestion> for DbExecutor {
    type Result = Result<(), DieselError>;

    fn handle(&mut self, msg: NewQuestion, _ctx: &mut Self::Context) -> Self::Result {
        use schema::questions::dsl::questions;
        let connection: &MysqlConnection = &self.0.get().unwrap();

        diesel::insert_into(questions)
            .values(&msg)
            .execute(connection)
            .expect("Error saving the question");

        Ok(())
    }
}

impl Message for GetQuestion {
    type Result = Result<Questions, DieselError>;
}

impl Handler<GetQuestion> for DbExecutor {
    type Result = Result<Questions, DieselError>;

    fn handle(&mut self, msg: GetQuestion, _ctx: &mut Self::Context) -> Self::Result {
        use schema::questions::dsl::{id, questions};
        let connection: &MysqlConnection =
            &self.0.get().expect("Unable to get database connection.");

        let result: Questions = questions.filter(id.eq(&msg.0)).first(connection)?;

        Ok(result)
    }
}

impl Message for GetQuestionByPresentation {
    type Result = Result<Vec<Questions>, DieselError>;
}

impl Handler<GetQuestionByPresentation> for DbExecutor {
    type Result = Result<Vec<Questions>, DieselError>;

    fn handle(&mut self, msg: GetQuestionByPresentation, _ctx: &mut Self::Context) -> Self::Result {
        use schema::questions;
        use schema::questions::dsl::presentation_id;
        let connection: &MysqlConnection =
            &self.0.get().expect("Unable to get database connection.");

        let questions: Vec<Questions> = questions::table
            .filter(presentation_id.eq(msg.presentation_id))
            .load(connection)?;

        Ok(questions)
    }
}

/// `/questions` POST
///
/// Headers:
///
/// Content type: application/json
/// Authorization: token <access_token>
///
/// Body:
/// ```json
/// {
///    "title": "New Question",
///    "presentation_id": 1,
/// }
/// ```
///
/// Response: 200 OK
pub fn post(
    data: Json<NewQuestionJson>,
    state: State<AppState>,
    req: HttpRequest<AppState>,
) -> Box<Future<Item = HttpResponse, Error = Error>> {
    let gh_user_id_session = req
        .session()
        .get::<GitHubUserId>(GH_USER_SESSION_ID_KEY)
        .into_future();

    let now = Utc::now();

    gh_user_id_session
        .from_err()
        .and_then(move |gh_user_id| {
            let input = data.into_inner();
            let new_question = NewQuestion::new(
                input.title,
                now.naive_utc(),
                input.presentation_id,
                gh_user_id.unwrap().id,
            );

            state
                .db
                .send(new_question)
                .from_err()
                .and_then(|response| match response {
                    Ok(_) => Ok(HttpResponse::Ok().finish()),
                    Err(_) => Ok(HttpResponse::InternalServerError().into()),
                })
        })
        .responder()
}

/// `/questions/{id}` GET
///
/// Response:
/// ```json
/// {
///    "id": 23,
///    "title": "New Question",
///    "created": "2019-11-01T14:30:30",
///    "presentation_id": 3,
///    "user_id": 7,
/// }
/// ```
pub fn get(
    data: Path<GetQuestion>,
    req: HttpRequest<AppState>,
) -> Box<Future<Item = HttpResponse, Error = Error>> {
    let state: &AppState = req.state();

    state
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

/// Returns questions for a presentation.
///
/// `/questions-presentation` GET
///
/// Parameters:
///
/// presentation_id: <presentation_id>
///
/// Response:
/// ```json
/// [
///    {
///         "id": 23,
///         "title": "New Question",
///         "created": "2019-11-01T14:30:30",
///         "presentation_id": 3,
///         "user_id": 7,
///     }
/// ]
/// ```
pub fn get_by_presentation(
    data: Query<GetQuestionByPresentation>,
    req: HttpRequest<AppState>,
) -> Box<Future<Item = HttpResponse, Error = Error>> {
    let state: &AppState = req.state();

    state
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
