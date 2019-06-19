use actix::{Handler, Message};
use actix_web::middleware::session::RequestSession;
use actix_web::AsyncResponder;
use actix_web::Error;
use actix_web::{HttpRequest, HttpResponse, Json, State};
use chrono::Utc;
use diesel::query_dsl::RunQueryDsl;
use diesel::result::Error as DieselError;
use diesel::MysqlConnection;
use futures::future::IntoFuture;
use futures::Future;
use middleware::GitHubUserId;
use models::{NewOption, NewOptionJson};
use GH_USER_SESSION_ID_KEY;
use {AppState, DbExecutor};

impl Message for NewOption {
    type Result = Result<(), DieselError>;
}

impl Handler<NewOption> for DbExecutor {
    type Result = Result<(), DieselError>;

    fn handle(&mut self, msg: NewOption, _ctx: &mut Self::Context) -> Self::Result {
        use schema::options::dsl::options;
        let connection: &MysqlConnection = &self.0.get().unwrap();

        diesel::insert_into(options)
            .values(&msg)
            .execute(connection)
            .expect("Error saving the option.");

        Ok(())
    }
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
pub fn post(
    data: Json<NewOptionJson>,
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
            let new_option = NewOption::new(
                input.data,
                gh_user_id.unwrap().id,
                input.question_id,
                now.naive_utc(),
            );

            state
                .db
                .send(new_option)
                .from_err()
                .and_then(|response| match response {
                    Ok(_) => Ok(HttpResponse::Ok().finish()),
                    Err(_) => Ok(HttpResponse::InternalServerError().into()),
                })
        })
        .responder()
}
