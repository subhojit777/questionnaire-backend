use crate::models::{NewOption, NewOptionJson, Option};
use crate::DbPool;

use crate::session::get_user_by_name;
use actix_identity::Identity;
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
///
/// Cookies:
///
/// auth-cookie: <cookie_value>
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
pub async fn post(
    pool: Data<DbPool>,
    data: Json<NewOptionJson>,
    id: Identity,
) -> Result<HttpResponse, Error> {
    if let Some(name) = id.identity() {
        let connection = pool.get().expect("Could not get database connection.");

        let user = block(move || get_user_by_name(name, &connection))
            .await
            .map_err(|_| {
                HttpResponse::InternalServerError()
                    .body("Something went wrong while retrieving the user.")
            })?;

        let input = data.into_inner();
        let now = Utc::now();
        // TODO: Try not to retrieve the connection again.
        let connection = pool.get().expect("unable to get database connection.");
        let record = NewOption::new(input.data, user.id, input.question_id, now.naive_utc());

        block(move || new_option(record, &connection))
            .await
            .map_err(|_| HttpResponse::InternalServerError().finish())?;

        Ok(HttpResponse::Ok().finish())
    } else {
        Ok(HttpResponse::BadRequest().body("Could not identify user."))
    }
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
