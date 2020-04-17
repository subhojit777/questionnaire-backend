use crate::questions::get_question_by_presentation;
use crate::DbPool;
use actix::prelude::*;
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
use std::time::{Duration, Instant};

const HEARTBEAT_INTERVAL: Duration = Duration::from_secs(5);
type PooledDatabaseConnection = PooledConnection<ConnectionManager<MysqlConnection>>;

struct WebSocket {
    heart_beat: Instant,
    db_connection: PooledDatabaseConnection,
}

#[derive(Deserialize)]
struct WebSocketRequest {
    presentation_id: i32,
    question_index: usize,
}

#[derive(Serialize)]
struct WebSocketResponse {
    new_question_index: usize,
}

impl WebSocket {
    pub fn new(db_connection: PooledDatabaseConnection) -> Self {
        Self {
            heart_beat: Instant::now(),
            db_connection,
        }
    }

    pub fn heart_beat(&self, ctx: &mut <Self as Actor>::Context) {
        ctx.run_interval(HEARTBEAT_INTERVAL, |_, ctx| {
            ctx.ping(b"");
        });
    }
}

impl Actor for WebSocket {
    type Context = WebsocketContext<Self>;

    fn started(&mut self, ctx: &mut Self::Context) {
        self.heart_beat(ctx);
    }
}

impl StreamHandler<Result<ws::Message, ProtocolError>> for WebSocket {
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

                let next_question_index = message.question_index + 1;
                let mut new_question_index: usize = 0;
                if let Some(_) = questions.get(next_question_index) {
                    new_question_index = next_question_index;
                }

                let response = WebSocketResponse { new_question_index };
                ctx.text(serde_json::to_string(&response).expect("Could not parse to JSON."));
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
    let response = ws::start(WebSocket::new(connection), &request, stream);
    response
}
