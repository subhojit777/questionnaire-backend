use crate::models::{NewQuestion, NewQuestionJson, Questions};
use crate::DbPool;

use crate::session::get_user_by_name;
use actix_identity::Identity;
use actix_web::web::{block, Data, Json, Path};
use actix_web::{get, post};
use actix_web::{Error, HttpResponse};
use chrono::Utc;
use diesel::prelude::*;
use diesel::result::Error as DieselError;

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

fn get_question_by_presentation(
    presentation_id: i32,
    connection: &MysqlConnection,
) -> Result<Vec<Questions>, DieselError> {
    use crate::schema::questions;
    use crate::schema::questions::dsl::{created, presentation_id as pid};

    let questions: Vec<Questions> = questions::table
        .filter(pid.eq(presentation_id))
        .order_by(created.asc())
        .load(connection)?;

    Ok(questions)
}

/// `/questions` POST
///
/// Headers:
///
/// Content type: application/json
///
/// Cookies:
///
/// auth-cookie: <cookie_value>
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
pub async fn post(
    pool: Data<DbPool>,
    data: Json<NewQuestionJson>,
    id: Identity,
) -> Result<HttpResponse, Error> {
    if let Some(name) = id.identity() {
        let connection = pool.get().expect("Unable to get database connection.");

        let user = block(move || get_user_by_name(name, &connection))
            .await
            .map_err(|_| HttpResponse::InternalServerError().body("Could not find user."))?;

        let now = Utc::now();
        let input = data.into_inner();
        let record = NewQuestion::new(input.title, now.naive_utc(), input.presentation_id, user.id);
        // TODO: Try not to retrieve the connection again.
        let connection = pool.get().expect("Unable to get database connection.");

        block(move || new_question(record, &connection))
            .await
            .map_err(|_| {
                HttpResponse::InternalServerError().body("Could not create new question.")
            })?;

        Ok(HttpResponse::Ok().finish())
    } else {
        return Ok(HttpResponse::BadRequest().body("could not identify the user."));
    }
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
/// `/questions-presentation/{presentation_id}` GET
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
#[get("/questions-presentation/{id}")]
pub async fn get_by_presentation(
    pool: Data<DbPool>,
    data: Path<i32>,
) -> Result<HttpResponse, Error> {
    let connection = pool.get().expect("Could not connect to database");
    let presentation_id = data.into_inner();

    let results = block(move || get_question_by_presentation(presentation_id, &connection))
        .await
        .map_err(|_| HttpResponse::InternalServerError().finish())?;

    Ok(HttpResponse::Ok().json(results))
}
