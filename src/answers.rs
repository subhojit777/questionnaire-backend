use crate::DbPool;
use actix_web::Error;

use crate::models::{Answer, AnswerInput, NewAnswer};
use crate::session::get_user_by_name;
use actix_identity::Identity;
use actix_web::web::{block, Data, Json, Path};
use actix_web::HttpResponse;
use actix_web::{get, post};
use chrono::Utc;
use diesel::prelude::*;
use diesel::result::Error as DieselError;

fn new_answer(
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
    data: Json<AnswerInput>,
    id: Identity,
) -> Result<HttpResponse, Error> {
    if let Some(name) = id.identity() {
        let connection = pool.get().expect("Could not get database connection.");

        let user = block(move || get_user_by_name(name, &connection))
            .await
            .map_err(|_| {
                HttpResponse::InternalServerError().body("Unable to retrieve user by name.")
            })?;

        // TODO: Try not to retrieve the connection again.
        let connection = pool.get().expect("couldn't get db connection from pool");
        let input = data.into_inner();

        block(move || new_answer(input.option_id, user.id, &connection))
            .await
            .map_err(|_| HttpResponse::InternalServerError().finish())?;

        Ok(HttpResponse::Ok().finish())
    } else {
        Ok(HttpResponse::BadRequest().body("Could not identify the user."))
    }
}

fn get_answer_by_id(answer_id: i32, connection: &MysqlConnection) -> Result<Answer, DieselError> {
    use crate::schema::answers::dsl::{answers, id};

    answers.filter(id.eq(answer_id)).first::<Answer>(connection)
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
#[get("/answers/{id}")]
pub async fn get(pool: Data<DbPool>, data: Path<i32>) -> Result<HttpResponse, Error> {
    let answer_id = data.into_inner();
    let connection = pool.get().expect("couldn't get db connection from pool.");

    let answer = block(move || get_answer_by_id(answer_id, &connection))
        .await
        .map_err(|_| HttpResponse::InternalServerError().finish())?;

    Ok(HttpResponse::Ok().json(answer))
}

fn get_answer_by_option_id(
    option_id: i32,
    connection: &MysqlConnection,
) -> Result<Vec<Answer>, DieselError> {
    use crate::schema::answers;
    use crate::schema::answers::dsl::option_id as schema_option_id;

    answers::table
        .filter(schema_option_id.eq(option_id))
        .load(connection)
}

/// Returns answers for an option.
///
/// `/answers-option/{option_id}` GET
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
#[get("/answers-option/{id}")]
pub async fn get_by_option(pool: Data<DbPool>, data: Path<i32>) -> Result<HttpResponse, Error> {
    let option_id = data.into_inner();
    let connection = pool.get().expect("unable to get database connection");

    let answers = block(move || get_answer_by_option_id(option_id, &connection))
        .await
        .map_err(|_| HttpResponse::InternalServerError().finish())?;

    Ok(HttpResponse::Ok().json(answers))
}
