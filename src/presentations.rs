use crate::middleware::GitHubUserId;
use crate::models::{NewPresentation, Presentation, PresentationInput};
use crate::DbPool;
use actix::{Handler, Message};
use actix_web::web::{block, Data, Json, Path};
use actix_web::Error;
use actix_web::{get, post};
use actix_web::{HttpRequest, HttpResponse};
use chrono::Utc;
use diesel::prelude::*;
use diesel::result::Error as DieselError;
use futures::Future;
use futures::IntoFuture;
use serde_json::ser::State;

fn new_presentation(
    data: NewPresentation,
    connection: &MysqlConnection,
) -> Result<(), DieselError> {
    use crate::schema::presentations::dsl::presentations;

    diesel::insert_into(presentations)
        .values(data)
        .execute(connection)
        .expect("Error saving the presentation");

    Ok(())
}

fn get_presentation(
    presentation_id: i32,
    connection: &MysqlConnection,
) -> Result<Presentation, DieselError> {
    use crate::schema::presentations::dsl::{id, presentations};

    let result: Presentation = presentations
        .filter(id.eq(presentation_id))
        .first(connection)?;

    Ok(result)
}

/// `/presentations` POST
///
/// Headers:
///
/// Content type: application/json
/// Authorization: token <access_token>
///
/// Body:
/// ```json
/// {
///    "title": "New Presentation"
/// }
/// ```
///
/// Response: 200 OK
#[post("/presentations")]
pub async fn post(
    pool: Data<DbPool>,
    data: Json<PresentationInput>,
) -> Result<HttpResponse, Error> {
    // TODO: Implement retrieval of user_id from session.
    let gh_user_id_session = Some(GitHubUserId { id: 1 });
    let now = Utc::now();
    let input = data.into_inner();

    return if let Some(user_id) = gh_user_id_session {
        let record = NewPresentation::new(input.title, user_id.id, now.naive_utc());
        let connection = pool.get().expect("Unable to get database connection.");

        block(move || new_presentation(record, &connection))
            .await
            .map_err(|_| HttpResponse::InternalServerError().finish());

        Ok(HttpResponse::Ok().finish())
    } else {
        Ok(HttpResponse::BadRequest().finish())
    };
}

/// `/presentations/{id}` GET
///
/// Response:
/// ```json
/// {
///    "id": 47,
///    "title": "New Presentation",
///    "user_id": 7,
///    "created": "2019-11-01T14:30:30"
/// }
/// ```
#[get("/presentations/{id}")]
pub async fn get(pool: Data<DbPool>, data: Path<i32>) -> Result<HttpResponse, Error> {
    let connection = pool.get().expect("Unable to get database connection.");
    let presentation_id = data.into_inner();

    let result = block(move || get_presentation(presentation_id, &connection))
        .await
        .map_err(|_| HttpResponse::InternalServerError().finish())?;

    Ok(HttpResponse::Ok().json(result))
}
