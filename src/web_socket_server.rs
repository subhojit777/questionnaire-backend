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

#[derive(Clone, Message)]
#[rtype(result = "()")]
pub struct RemoveSession(pub usize);

#[derive(Default)]
pub struct WebSocketServer {
    sessions: HashMap<usize, Recipient<Message>>,
    rng: ThreadRng,
}

impl WebSocketServer {
    pub fn send_message(&mut self, message: String) {
        for (_id, recipient) in &self.sessions {
            // TODO: This errors if a participant screen is refreshed.
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

    pub fn remove_session(&mut self, session_id: usize) {
        self.sessions.remove(&session_id);
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

impl Handler<RemoveSession> for WebSocketServer {
    type Result = MessageResult<RemoveSession>;

    fn handle(&mut self, msg: RemoveSession, _ctx: &mut Self::Context) -> Self::Result {
        let RemoveSession(id) = msg;

        self.remove_session(id);

        MessageResult(())
    }
}

impl SystemService for WebSocketServer {}
impl Supervised for WebSocketServer {}
