use crate::middleware::GitHubUserId;
use crate::models::{NewQuestion, NewQuestionJson, Questions};
use crate::DbPool;
use actix::{Handler, Message};
use actix_web::web::{block, Data, Json, Path, Query};
use actix_web::{Error, HttpRequest, HttpResponse};
use chrono::Utc;
use diesel::prelude::*;
use diesel::result::Error as DieselError;
use futures::Future;
use futures::IntoFuture;
use middleware::GitHubUserId;
use models::{GetQuestion, GetQuestionByPresentation, NewQuestion, NewQuestionJson, Questions};
use serde_json::ser::State;
use GH_USER_SESSION_ID_KEY;
use {AppState, DbExecutor};

fn new_question(input: NewQuestion, connection: &MysqlConnection) -> Result<(), DieselError> {
    use crate::schema::questions::dsl::questions;

    diesel::insert_into(questions)
        .values(input)
        .execute(connection)
        .expect("Error saving the question");

    Ok(())
}

fn get_question(question_id: i32, connection: &MysqlConnection) -> Result<Questions, DieselError> {
    use crate::schema::questions::dsl::{id, questions};

    let result: Questions = questions.filter(id.eq(question_id)).first(connection)?;

    Ok(result)
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
#[post("/questions")]
pub async fn post(pool: Data<DbPool>, data: Json<NewQuestionJson>) -> Result<HttpResponse, Error> {
    // TODO: Implement retrieval of user_id from session.
    let gh_user_id_session = Some(GitHubUserId { id: 1 });
    let now = Utc::now();
    let input = data.into_inner();

    return if let Some(user_id) = gh_user_id_session {
        let record = NewQuestion::new(
            input.title,
            now.naive_utc(),
            input.presentation_id,
            user_id.id,
        );
        let connection = pool.get().expect("Unable to get database connection.");

        block(move || new_question(record, &connection))
            .await
            .map_err(|_| HttpResponse::InternalServerError().finish())?;

        Ok(HttpResponse::Ok().finish())
    } else {
        Ok(HttpResponse::BadRequest().finish())
    };
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
#[get("/questions/{id}")]
pub async fn get(pool: Data<DbPool>, data: Path<i32>) -> Result<HttpResponse, Error> {
    let connection = pool.get().expect("Unable to get database connection.");
    let question_id = data.into_inner();

    let result = block(move || get_question(question_id, &connection))
        .await
        .map_err(|_| HttpResponse::InternalServerError().finish())?;

    Ok(HttpResponse::Ok().json(result))
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
