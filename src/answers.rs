use actix_web::{
    actix::{Handler, Message},
    error::Error,
    AsyncResponder, FutureResponse, HttpResponse, Json, State, HttpRequest,
};
use crate::*;
use diesel::prelude::*;
use futures::Future;
use models::{Answer, AnswerForm};
use oxide_auth::{frontends::actix::*, code_grant::frontend::OAuthError};

pub fn post(req: &HttpRequest<AppState>) -> Box<Future<Item = HttpResponse, Error = Error>> {
    let state: AppState = req.state().clone();
    Box::new(req.oauth2()
        .guard()
        .and_then(move |request| state.endpoint.send(request)
            .map_err(|_| OAuthError::InvalidRequest)
            .and_then(|result| result)
        )
        .map(|()|
            HttpResponse::Ok()
                .content_type("text/plain")
                .body("this should create new answer"))
        .or_else(|error| {
            Ok(ResolvedResponse::response_or_error((error))
                .actix_response()
                .into_builder()
                .content_type("text/plain")
                .body("something wrong happened"))
        }))
//        .json()
//        .from_err()
//        .and_then(|answer: AnswerForm| {
//            state.db.send(answer)
//                .from_err()
//                .and_then(|response| match response {
//                    Ok(result) => Ok(HttpResponse::Ok().json(result)),
//                    Err(_) => Ok(HttpResponse::InternalServerError().into()),
//                })
//        })
//        .or_else(|error| {
//            Ok(ResolvedResponse::response_or_error(error)
//                .actix_response()
//                .into_builder()
//                .content_type("text/plain")
//                .body("something wrong happened"))
//        })
    // let answer =answer.into_inner();

    // state
    //     .db
    //     .send(answer)
    //     .from_err()
    //     .and_then(|response| match response {
    //         Ok(result) => Ok(HttpResponse::Ok().json(result)),
    //         Err(_) => Ok(HttpResponse::InternalServerError().into()),
    //     }).responder()
}

impl Message for AnswerForm {
    type Result = Result<Answer, Error>;
}

impl Handler<AnswerForm> for DbExecutor {
    type Result = Result<Answer, Error>;

    fn handle(&mut self, msg: AnswerForm, _: &mut Self::Context) -> Self::Result {
        use schema::answers::dsl::{answers, question_id, user_id};

        let connection: &MysqlConnection = &self.0.get().unwrap();

        diesel::insert_into(answers)
            .values(&msg)
            .execute(connection)
            .expect("Error saving theanswer");

        let result: Answer = answers
            .filter(question_id.eq(&msg.question_id))
            .filter(user_id.eq(&msg.user_id))
            .first(connection)
            .unwrap();

        Ok(result)
    }
}
