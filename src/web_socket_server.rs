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

#[derive(Clone, Message)]
#[rtype(result = "usize")]
pub struct JoinSession(pub Recipient<Message>);

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

    pub fn add_session(&mut self, client: Recipient<Message>) -> usize {
        let id: usize = self.rng.gen();

        self.sessions.insert(id, client);

        id
    }
}

impl Actor for WebSocketServer {
    type Context = Context<Self>;

    fn started(&mut self, ctx: &mut Self::Context) {
        self.subscribe_system_async::<SendMessage>(ctx);
    }
}

impl Handler<SendMessage> for WebSocketServer {
    type Result = ();

    fn handle(&mut self, msg: SendMessage, _ctx: &mut Self::Context) {
        self.send_message(msg.content);
    }
}

impl Handler<JoinSession> for WebSocketServer {
    type Result = MessageResult<JoinSession>;

    fn handle(&mut self, msg: JoinSession, _ctx: &mut Self::Context) -> Self::Result {
        let JoinSession(client) = msg;

        let id = self.add_session(client);

        MessageResult(id)
    }
}

impl SystemService for WebSocketServer {}
impl Supervised for WebSocketServer {}
