use crate::middleware::GitHubUserId;
use crate::models::{NewOption, NewOptionJson, Option};
use crate::DbPool;

use actix_web::web::{block, Data, Json, Path};
use actix_web::Error;
use actix_web::HttpResponse;
use actix_web::{get, post};
use chrono::Utc;
use diesel::prelude::*;
use diesel::query_dsl::RunQueryDsl;
use diesel::result::Error as DieselError;
use diesel::MysqlConnection;

fn new_option(record: NewOption, connection: &MysqlConnection) -> Result<(), DieselError> {
    use crate::schema::options::dsl::options;

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

fn get_option_by_question_id(
    id: i32,
    connection: &MysqlConnection,
) -> Result<Vec<Option>, DieselError> {
    use crate::schema::options;
    use crate::schema::options::dsl::question_id;

    let options = options::table.filter(question_id.eq(id)).load(connection)?;

    Ok(options)
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
            .map_err(|_| HttpResponse::InternalServerError().finish())?;

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
/// `/options-question/{question_id}` GET
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
#[get("/options-question/{id}")]
pub async fn get_by_question(pool: Data<DbPool>, data: Path<i32>) -> Result<HttpResponse, Error> {
    let connection = pool.get().expect("unable to get database connection.");
    let question_id = data.into_inner();

    let results = block(move || get_option_by_question_id(question_id, &connection))
        .await
        .map_err(|_| HttpResponse::InternalServerError().finish())?;

    Ok(HttpResponse::Ok().json(results))
}
