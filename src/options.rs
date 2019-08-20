use actix::{Handler, Message};
use actix_web::middleware::session::RequestSession;
use actix_web::{AsyncResponder, Path};
use actix_web::{Error, Query};
use actix_web::{HttpRequest, HttpResponse, Json, State};
use chrono::Utc;
use diesel::prelude::*;
use diesel::query_dsl::RunQueryDsl;
use diesel::result::Error as DieselError;
use diesel::MysqlConnection;
use futures::future::IntoFuture;
use futures::Future;
use middleware::GitHubUserId;
use models::{GetOption, GetOptionsByQuestion, NewOption, NewOptionJson, Option};
use GH_USER_SESSION_ID_KEY;
use {AppState, DbExecutor};

impl Message for NewOption {
    type Result = Result<(), DieselError>;
}

impl Handler<NewOption> for DbExecutor {
    type Result = Result<(), DieselError>;

    fn handle(&mut self, msg: NewOption, _ctx: &mut Self::Context) -> Self::Result {
        use schema::options::dsl::options;
        let connection: &MysqlConnection =
            &self.0.get().expect("Unable to get database connection");

        diesel::insert_into(options)
            .values(&msg)
            .execute(connection)
            .expect("Error saving the option.");

        Ok(())
    }
}

impl Message for GetOption {
    type Result = Result<Option, DieselError>;
}

impl Handler<GetOption> for DbExecutor {
    type Result = Result<Option, DieselError>;

    fn handle(&mut self, msg: GetOption, _ctx: &mut Self::Context) -> Self::Result {
        use schema::options::dsl::{id, options};
        let connection: &MysqlConnection =
            &self.0.get().expect("Unable to get database connection.");

        let result: Option = options.filter(id.eq(&msg.0)).first(connection)?;

        Ok(result)
    }
}

impl Message for GetOptionsByQuestion {
    type Result = Result<Vec<Option>, DieselError>;
}

impl Handler<GetOptionsByQuestion> for DbExecutor {
    type Result = Result<Vec<Option>, DieselError>;

    fn handle(&mut self, msg: GetOptionsByQuestion, _ctx: &mut Self::Context) -> Self::Result {
        use schema::options;
        use schema::options::dsl::question_id;

        let connection: &MysqlConnection =
            &self.0.get().expect("Unable to get database connection.");

        let options: Vec<Option> = options::table
            .filter(question_id.eq(msg.question_id))
            .load(connection)?;

        Ok(options)
    }
}

/// `/options` POST
///
/// Headers:
///
/// Content type: application/json
/// Authorization: token <access_token>
///
/// Body:
/// ```json
/// {
///    "data": "Option 1",
///    "question_id": 1,
/// }
/// ```
///
/// Response: 200 OK
pub fn post(
    data: Json<NewOptionJson>,
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
            let new_option = NewOption::new(
                input.data,
                gh_user_id.unwrap().id,
                input.question_id,
                now.naive_utc(),
            );

            state
                .db
                .send(new_option)
                .from_err()
                .and_then(|response| match response {
                    Ok(_) => Ok(HttpResponse::Ok().finish()),
                    Err(_) => Ok(HttpResponse::InternalServerError().into()),
                })
        })
        .responder()
}

/// `/options/{id}` GET
///
/// Headers:
///
/// Authorization: token <access_token>
///
/// Response:
/// ```json
/// {
///    "id": 12,
///    "data": "Option 1",
///    "user_id": 9,
///    "question_id": 1,
///    "created": "2019-06-19T03:40:50"
/// }
/// ```
pub fn get(
    data: Path<GetOption>,
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

/// Returns options for a question.
///
/// `/options-question` GET
///
/// Parameters:
///
/// question_id: {id}
///
/// Headers:
///
/// Authorization: token <access_token>
///
/// Response:
/// ```json
/// [
///    {
///         "id": 12,
///         "data": "Option 1",
///         "user_id": 9,
///         "question_id": 1,
///         "created": "2019-06-19T03:40:50"
///     }
/// ]
/// ```
pub fn get_by_question(
    data: Query<GetOptionsByQuestion>,
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
