use actix::prelude::*;
use rand::prelude::ThreadRng;
use rand::Rng;
use std::collections::HashMap;

#[derive(Message)]
#[rtype(result = "()")]
pub struct Message(pub String);

pub struct WebSocketServer {
    sessions: HashMap<usize, Recipient<Message>>,
    rng: ThreadRng,
}

impl Default for WebSocketServer {
    fn default() -> Self {
        WebSocketServer {
            sessions: HashMap::new(),
            rng: rand::thread_rng(),
        }
    }
}

impl WebSocketServer {
    pub fn send_message(&self, message: &str, skip_id: usize) {
        for (id, recipient) in &self.sessions {
            if *id != skip_id {
                let _ = recipient.do_send(Message(message.to_owned()));
            }
        }
    }
}

#[derive(Message)]
#[rtype(result = "()")]
pub struct ClientMessage {
    pub id: usize,
    pub msg: String,
}

impl Actor for WebSocketServer {
    type Context = Context<Self>;
}

#[derive(Message)]
#[rtype(usize)]
pub struct Connect {
    pub addr: Recipient<Message>,
}

impl Handler<Connect> for WebSocketServer {
    type Result = usize;

    fn handle(&mut self, msg: Connect, _ctx: &mut Context<Self>) -> Self::Result {
        let id = self.rng.gen::<usize>();
        // TODO: Not getting connected.
        self.sessions.insert(id, msg.addr);

        id
    }
}

impl Handler<ClientMessage> for WebSocketServer {
    type Result = ();

    fn handle(&mut self, msg: ClientMessage, _: &mut Context<Self>) -> Self::Result {
        self.send_message(msg.msg.as_str(), msg.id);
    }
}
