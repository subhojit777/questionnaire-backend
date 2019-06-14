use actix::{Handler, Message};
use actix_web::middleware::session::RequestSession;
use actix_web::AsyncResponder;
use actix_web::{Error, HttpRequest, HttpResponse, Json, State};
use chrono::Utc;
use diesel::query_dsl::RunQueryDsl;
use diesel::result::Error as DieselError;
use diesel::MysqlConnection;
use futures::Future;
use futures::IntoFuture;
use middleware::GitHubUserId;
use models::{NewQuestion, NewQuestionJson};
use GH_USER_SESSION_ID_KEY;
use {AppState, DbExecutor};

impl Message for NewQuestion {
    type Result = Result<(), DieselError>;
}

impl Handler<NewQuestion> for DbExecutor {
    type Result = Result<(), DieselError>;

    fn handle(&mut self, msg: NewQuestion, _ctx: &mut Self::Context) -> Self::Result {
        use schema::questions::dsl::questions;
        let connection: &MysqlConnection = &self.0.get().unwrap();

        diesel::insert_into(questions)
            .values(&msg)
            .execute(connection)
            .expect("Error saving the question");

        Ok(())
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
pub fn post(
    data: Json<NewQuestionJson>,
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
            let new_question = NewQuestion::new(
                input.title,
                now.naive_utc(),
                input.presentation_id,
                gh_user_id.unwrap().id,
            );

            state
                .db
                .send(new_question)
                .from_err()
                .and_then(|response| match response {
                    Ok(_) => Ok(HttpResponse::Ok().finish()),
                    Err(_) => Ok(HttpResponse::InternalServerError().into()),
                })
        })
        .responder()
}
