use actix::prelude::*;
use actix_http::ws::ProtocolError;
use actix_web::web::Payload;
use actix_web::HttpRequest;
use actix_web::{get, HttpResponse};
use actix_web_actors::ws;
use actix_web_actors::ws::WebsocketContext;
use std::time::Duration;

const HEARTBEAT_INTERVAL: Duration = Duration::from_secs(5);

struct WebSocket {
    presentation_id: i32,
    question_index: i32,
}

impl WebSocket {
    pub fn new(presentation_id: i32, question_index: i32) -> Self {
        Self {
            presentation_id,
            question_index,
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
            Ok(ws::Message::Ping(msg)) => ctx.pong(&msg),
            _ => ctx.stop(),
        }
    }
}

#[get("/ws/")]
pub async fn index(
    request: HttpRequest,
    stream: Payload,
) -> Result<HttpResponse, actix_web::Error> {
    let response = ws::start(WebSocket::new(-1, -1), &request, stream);
    response
}
