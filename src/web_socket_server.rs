use actix::prelude::*;
use actix_broker::BrokerSubscribe;
use rand::prelude::ThreadRng;
use rand::Rng;
use std::collections::HashMap;

#[derive(Message)]
#[rtype(result = "()")]
pub struct Message(pub String);

#[derive(Message, Clone)]
#[rtype(result = "()")]
pub struct SendMessage {
    pub name: String,
    pub id: usize,
    pub content: String,
}

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
    pub fn send_message(&mut self, message: String) {
        for (_id, recipient) in &self.sessions {
            recipient
                .do_send(Message(message.to_owned()))
                .expect("Could not send message to the client.");
        }
    }
}

impl Actor for WebSocketServer {
    type Context = Context<Self>;

    fn started(&mut self, ctx: &mut Self::Context) {
        self.subscribe_system_async::<SendMessage>(ctx);
    }
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

impl Handler<SendMessage> for WebSocketServer {
    type Result = ();

    fn handle(&mut self, msg: SendMessage, _ctx: &mut Self::Context) {
        self.send_message(msg.content);
    }
}

impl SystemService for WebSocketServer {}
impl Supervised for WebSocketServer {}
