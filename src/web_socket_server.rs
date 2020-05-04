use actix::prelude::*;
use rand::prelude::ThreadRng;
use rand::Rng;
use std::collections::HashMap;

pub struct WebSocketServer {
    // TODO: This will error on build. Continue from here.
    sessions: HashMap<usize, Recipient<dyn Message>>,
    rng: ThreadRng,
}

impl Actor for WebSocketServer {
    type Context = Context<Self>;
}

pub struct Connect {
    pub addr: Recipient<dyn Message>,
}

impl Handler<Connect> for WebSocketServer {
    type Result = usize;

    fn handle(&mut self, msg: Connect, _ctx: &mut Context<Self>) -> Self::Result {
        let id = self.rng.gen::<usize>();
        self.sessions.insert(id, msg.addr);

        id
    }
}
