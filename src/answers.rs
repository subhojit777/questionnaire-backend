use crate::{AppState, DbExecutor};
use actix_web::client::ClientResponse;
use actix_web::error as AWError;
use actix_web::http::HeaderMap;
use actix_web::http::StatusCode;
use actix_web::ResponseError;
use actix_web::{
    actix::{Handler, Message},
    client, AsyncResponder, HttpRequest, HttpResponse, Json, State,
};
use diesel::prelude::*;
use futures::Async;
use futures::Future;
use models::{Answer, AnswerForm};
use std::error as StdError;
use std::fmt;
use std::fmt::Formatter;

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
                .and_then(|res: ClientResponse| {
                    if res.status() == 200 {
                        return Ok(HttpResponse::Ok().body("answers get"));
                    }

                    Ok(HttpResponse::Forbidden().into())
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

#[derive(Debug)]
struct OauthError {
    name: &'static str,
}

impl fmt::Display for OauthError {
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
        write!(f, "{}", self.name)
    }
}

impl StdError::Error for OauthError {
    fn description(&self) -> &str {
        self.name
    }
}

impl ResponseError for OauthError {
    fn error_response(&self) -> HttpResponse {
        HttpResponse::new(StatusCode::INTERNAL_SERVER_ERROR)
    }
}

struct HeaderMapWrapper {
    map: HeaderMap,
}

impl Future for HeaderMapWrapper {
    type Item = String;
    type Error = OauthError;

    fn poll(&mut self) -> Result<Async<Self::Item>, Self::Error> {
        if let Some(token) = self.map.get("authorization") {
            Ok(Async::Ready(token.to_str().unwrap().to_string()))
        } else {
            Err(OauthError {
                name: "token not found",
            })
        }
    }
}
