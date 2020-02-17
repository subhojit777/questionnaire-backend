use actix::{Handler, Message};
use actix_web::web::{Json, Path};
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

impl Message for NewPresentation {
    type Result = Result<(), error::Db>;
}

impl Handler<NewPresentation> for DbExecutor {
    type Result = Result<(), error::Db>;

    fn handle(&mut self, msg: NewPresentation, _ctx: &mut Self::Context) -> Self::Result {
        use schema::presentations::dsl::presentations;

        let connection: &MysqlConnection = &self.0.get().unwrap();

        diesel::insert_into(presentations)
            .values(&msg)
            .execute(connection)
            .expect("Error saving the presentation");

        Ok(())
    }
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
pub fn post(
    data: Json<PresentationInput>,
    state: State,
    req: HttpRequest,
) -> Box<dyn Future<Item = HttpResponse, Error = Error>> {
    let gh_user_id_session = req
        .session()
        .get::<GitHubUserId>(GH_USER_SESSION_ID_KEY)
        .into_future();

    let now = Utc::now();

    gh_user_id_session
        .from_err()
        .and_then(move |gh_user_id| {
            let input = data.into_inner();
            let new_presentation =
                NewPresentation::new(input.title, gh_user_id.unwrap().id, now.naive_utc());

            state
                .db
                .send(new_presentation)
                .from_err()
                .and_then(|response| match response {
                    Ok(_) => Ok(HttpResponse::Ok().finish()),
                    Err(_) => Ok(HttpResponse::InternalServerError().into()),
                })
        })
        .responder()
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
