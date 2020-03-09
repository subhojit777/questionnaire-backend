use crate::middleware::GitHubUserId;
use crate::models::{NewPresentation, PresentationInput};
use crate::DbPool;
use actix::{Handler, Message};
use actix_web::web::{block, Data, Json, Path};
use actix_web::Error;
use actix_web::{HttpRequest, HttpResponse};
use chrono::Utc;
use diesel::prelude::*;
use diesel::result::Error as DieselError;
use futures::Future;
use futures::IntoFuture;
use middleware::GitHubUserId;
use models::{GetPresentation, NewPresentation, Presentation, PresentationInput};
use serde_json::ser::State;
use GH_USER_SESSION_ID_KEY;
use {error, DbExecutor};

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

impl Message for GetPresentation {
    type Result = Result<Presentation, DieselError>;
}

impl Handler<GetPresentation> for DbExecutor {
    type Result = Result<Presentation, DieselError>;

    fn handle(&mut self, msg: GetPresentation, _ctx: &mut Self::Context) -> Self::Result {
        use schema::presentations::dsl::{id, presentations};

        let connection: &MysqlConnection = &self.0.get().unwrap();

        let result: Presentation = presentations.filter(id.eq(&msg.0)).first(connection)?;

        Ok(result)
    }
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
pub fn get(
    data: Path<GetPresentation>,
    req: HttpRequest,
) -> Box<dyn Future<Item = HttpResponse, Error = Error>> {
    req.state()
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
