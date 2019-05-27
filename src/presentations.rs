use actix_web::actix::{Handler, Message};
use actix_web::middleware::session::RequestSession;
use actix_web::AsyncResponder;
use actix_web::Error;
use actix_web::{HttpRequest, HttpResponse, Json, State};
use chrono::Utc;
use diesel::query_dsl::RunQueryDsl;
use diesel::MysqlConnection;
use futures::Future;
use futures::IntoFuture;
use middleware::GitHubUserId;
use models::{NewPresentation, PresentationInput};
use {error, DbExecutor};
use {AppState, GH_USER_SESSION_ID_KEY};

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
