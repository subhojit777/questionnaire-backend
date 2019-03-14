use crate::{error::Oauth as OauthError, AppState, DbExecutor};
use actix_web::client::ClientResponse;
use actix_web::error as AWError;
use actix_web::http::HeaderMap;
use actix_web::http::StatusCode;
use actix_web::{
    actix::{Handler, Message},
    client, AsyncResponder, HttpRequest, HttpResponse, Json, State,
};
use diesel::prelude::*;
use futures::Async;
use futures::Future;
use models::{Answer, AnswerForm};

pub fn post(
    answer_form: Json<AnswerForm>,
    state: State<AppState>,
    _req: HttpRequest<AppState>,
) -> Box<Future<Item = HttpResponse, Error = AWError::Error>> {
    let answer = answer_form.into_inner();

    state
        .db
        .send(answer)
        .from_err()
        .and_then(|response| match response {
            Ok(result) => Ok(HttpResponse::Ok().json(result)),
            Err(_) => Ok(HttpResponse::InternalServerError().into()),
        })
        .responder()
}

/// TODO: This is not yet fully implemented. It is supposed to return an answer based on an ID.
/// The code inside explains how the oauth wrapper is supposed to work.
pub fn get(req: HttpRequest<AppState>) -> Box<Future<Item = HttpResponse, Error = AWError::Error>> {
    let header_map: HeaderMapWrapper = HeaderMapWrapper {
        map: req.headers().clone(),
    };

    header_map
        .from_err()
        .and_then(|access_token| {
            client::get("https://api.github.com/user")
                .header("Authorization", access_token)
                .finish()
                .unwrap()
                .send()
                .from_err()
                .and_then(|res: ClientResponse| match res.status() {
                    StatusCode::OK => Ok(HttpResponse::Ok().into()),
                    StatusCode::FORBIDDEN => Ok(HttpResponse::Forbidden().into()),
                    _ => Ok(HttpResponse::NotAcceptable().into()),
                })
        })
        .responder()
}

impl Message for AnswerForm {
    type Result = Result<Answer, AWError::Error>;
}

impl Handler<AnswerForm> for DbExecutor {
    type Result = Result<Answer, AWError::Error>;

    fn handle(&mut self, msg: AnswerForm, _: &mut Self::Context) -> Self::Result {
        use schema::answers::dsl::{answers, question_id, user_id};

        let connection: &MysqlConnection = &self.0.get().unwrap();

        diesel::insert_into(answers)
            .values(&msg)
            .execute(connection)
            .expect("Error saving the answer_form");

        let result: Answer = answers
            .filter(question_id.eq(&msg.question_id))
            .filter(user_id.eq(&msg.user_id))
            .first(connection)
            .unwrap();

        Ok(result)
    }
}

/// Future implementation of actix_web::http::HeaderMap.
struct HeaderMapWrapper {
    map: HeaderMap,
}

impl Future for HeaderMapWrapper {
    type Item = String;
    type Error = OauthError;

    fn poll(&mut self) -> Result<Async<Self::Item>, Self::Error> {
        if let Some(token) = self.map.get("authorization") {
            match token.to_str() {
                Ok(val) => return Ok(Async::Ready(val.to_string())),
                Err(_) => return Err(OauthError::BadRequest),
            };
        } else {
            Err(OauthError::BadRequest)
        }
    }
}
