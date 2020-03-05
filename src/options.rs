use crate::middleware::GitHubUserId;
use crate::models::{NewOption, NewOptionJson, Option};
use crate::DbPool;
use actix::{Handler, Message};
use actix_web::web::{block, Data, Json, Path, Query};
use actix_web::Error;
use actix_web::{get, post};
use actix_web::{HttpRequest, HttpResponse};
use chrono::Utc;
use diesel::prelude::*;
use diesel::query_dsl::RunQueryDsl;
use diesel::result::Error as DieselError;
use diesel::MysqlConnection;
use futures::future::IntoFuture;
use futures::Future;
use middleware::GitHubUserId;
use models::{GetOption, GetOptionsByQuestion, NewOption, NewOptionJson, Option};
use serde_json::ser::State;
use GH_USER_SESSION_ID_KEY;
use {AppState, DbExecutor};

fn new_option(record: NewOption, connection: &MysqlConnection) -> Result<(), DieselError> {
    use schema::options::dsl::options;

    diesel::insert_into(options)
        .values(&record)
        .execute(connection)
        .expect("Error saving the option.");

    Ok(())
}

fn get_option(option_id: i32, connection: &MysqlConnection) -> Result<Option, DieselError> {
    use crate::schema::options::dsl::{id, options};

    options.filter(id.eq(option_id)).first::<Option>(connection)
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
#[post("/options")]
pub async fn post(pool: Data<DbPool>, data: Json<NewOptionJson>) -> Result<HttpResponse, Error> {
    // TODO: Implement retrieval of user_id from session.
    let gh_user_id_session = Some(GitHubUserId { id: 1 });
    let input = data.into_inner();
    let now = Utc::now();

    return if let Some(user_id) = gh_user_id_session {
        let connection = pool.get().expect("unable to get database connection.");
        let record = NewOption::new(input.data, user_id.id, input.question_id, now.naive_utc());

        block(move || new_option(record, &connection))
            .await
            .map_err(|_| {
                return HttpResponse::InternalServerError().finish()?;
            });

        Ok(HttpResponse::Ok().finish())
    } else {
        Ok(HttpResponse::BadRequest().finish())
    };
}

/// `/options/{id}` GET
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
#[get("/options/{id}")]
pub async fn get(pool: Data<DbPool>, data: Path<i32>) -> Result<HttpResponse, Error> {
    let connection = pool.get().expect("unable to get database connection.");
    let option_id = data.into_inner();

    let result = block(move || get_option(option_id, &connection))
        .await
        .map_err(|_| HttpResponse::InternalServerError().finish())?;

    Ok(HttpResponse::Ok().json(result))
}

/// Returns options for a question.
///
/// `/options-question` GET
///
/// Parameters:
///
/// question_id: {id}
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
    req: HttpRequest,
) -> Box<dyn Future<Item = HttpResponse, Error = Error>> {
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
