use crate::questions::get_question_by_presentation;
use crate::web_socket_server::JoinSession;
use crate::web_socket_server::Message;
use crate::web_socket_server::RemoveSession;
use crate::web_socket_server::SendMessage;
use crate::web_socket_server::WebSocketServer;
use crate::DbPool;
use actix::prelude::*;
use actix_broker::BrokerIssue;
use actix_http::ws::ProtocolError;
use actix_web::web::{Data, Payload};
use actix_web::HttpRequest;
use actix_web::{get, HttpResponse};
use actix_web_actors::ws;
use actix_web_actors::ws::WebsocketContext;
use diesel::r2d2::ConnectionManager;
use diesel::r2d2::PooledConnection;
use diesel::MysqlConnection;
use serde::Deserialize;
use serde::Serialize;
use serde_json::json;
use std::time::Instant;

type PooledDatabaseConnection = PooledConnection<ConnectionManager<MysqlConnection>>;

#[derive(Deserialize)]
enum Direction {
    Forward,
    Backward,
}

struct WebSocketSession {
    id: usize,
    name: String,
    heart_beat: Instant,
    db_connection: PooledDatabaseConnection,
}

trait Event {
    fn parse_request(data: &str) -> Self;

    fn send_response(&self, connection: &PooledDatabaseConnection) -> String;
}

#[derive(Deserialize)]
struct NavigateEvent {
    presentation_id: i32,
    question_index: usize,
    direction: Direction,
}

impl Event for NavigateEvent {
    fn parse_request(data: &str) -> Self {
        serde_json::from_str(data).expect("Unable to parse navigation request.")
    }

    fn send_response(&self, connection: &PooledDatabaseConnection) -> String {
        let questions = get_question_by_presentation(self.presentation_id, connection)
            .expect("Unable to retrieve the questions for the presentation.");

        let mut new_question_index: usize = 0;
        let num_questions = questions.len();

        match self.direction {
            Direction::Forward => {
                let next_question_index = self.question_index + 1;
                if next_question_index < num_questions
                    && questions.get(next_question_index).is_some()
                {
                    new_question_index = next_question_index;
                }
            }
            Direction::Backward => {
                if self.question_index > 0 && questions.get(self.question_index).is_some() {
                    new_question_index = self.question_index - 1;
                }
            }
        }
        let response = json!({
            "new_question_index": new_question_index,
        });

        response.to_string()
    }
}

#[derive(Deserialize)]
struct WebSocketRequest {
    presentation_id: i32,
    question_index: usize,
    direction: Direction,
}

#[derive(Serialize)]
struct WebSocketResponse {
    new_question_index: usize,
}

impl WebSocketSession {
    pub fn new(db_connection: PooledDatabaseConnection) -> Self {
        Self {
            id: 0,
            name: "Main".to_owned(),
            heart_beat: Instant::now(),
            db_connection,
        }
    }

    pub fn send_msg(&self, msg: String) {
        let msg = SendMessage {
            name: self.name.clone(),
            id: self.id,
            content: msg,
        };

        self.issue_system_async(msg);
    }
}

impl Actor for WebSocketSession {
    type Context = WebsocketContext<Self>;

    fn started(&mut self, ctx: &mut Self::Context) {
        let join_session = JoinSession(ctx.address().recipient());
        WebSocketServer::from_registry()
            .send(join_session)
            .into_actor(self)
            .then(|id, act, _ctx| {
                if let Ok(id) = id {
                    act.id = id;
                }

                fut::ready(())
            })
            .wait(ctx);
    }

    fn stopped(&mut self, _ctx: &mut Self::Context) {
        let remove_session = RemoveSession(self.id);

        self.issue_system_async(remove_session);
    }
}

impl Handler<Message> for WebSocketSession {
    type Result = ();

    fn handle(&mut self, msg: Message, ctx: &mut Self::Context) {
        ctx.text(msg.0);
    }
}

impl StreamHandler<Result<ws::Message, ProtocolError>> for WebSocketSession {
    fn handle(&mut self, item: Result<ws::Message, ProtocolError>, ctx: &mut Self::Context) {
        match item {
            Ok(ws::Message::Ping(msg)) => {
                self.heart_beat = Instant::now();
                ctx.pong(&msg);
            }
            Ok(ws::Message::Pong(_msg)) => {
                self.heart_beat = Instant::now();
            }
            Ok(ws::Message::Text(text)) => {
                let connection = &self.db_connection;

                let message: WebSocketRequest = serde_json::from_str(&text)
                    .expect("Unable to parse the text message from web socket");

                let questions = get_question_by_presentation(message.presentation_id, connection)
                    .expect("Unable to retrieve the questions for the presentation.");

                let mut new_question_index: usize = 0;
                let num_questions = questions.len();

                match message.direction {
                    Direction::Forward => {
                        let next_question_index = message.question_index + 1;
                        if next_question_index < num_questions
                            && questions.get(next_question_index).is_some()
                        {
                            new_question_index = next_question_index;
                        }
                    }
                    Direction::Backward => {
                        if message.question_index > 0
                            && questions.get(message.question_index).is_some()
                        {
                            new_question_index = message.question_index - 1;
                        }
                    }
                }

                let response = WebSocketResponse { new_question_index };
                self.send_msg(serde_json::to_string(&response).expect("Could not parse to JSON."));
            }
            Ok(ws::Message::Binary(_)) => println!("Unexpected binary"),
            _ => ctx.stop(),
        }
    }
}

#[get("/ws/")]
pub async fn index(
    request: HttpRequest,
    stream: Payload,
    pool: Data<DbPool>,
) -> Result<HttpResponse, actix_web::Error> {
    let connection = pool.get().expect("unable to get database connection");
    let response = ws::start(WebSocketSession::new(connection), &request, stream);
    response
}
