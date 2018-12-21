extern crate chrono;
extern crate serde_json;
extern crate oxide_auth;
extern crate actix;
extern crate env_logger;
#[macro_use]
extern crate diesel;
extern crate actix_web;
extern crate dotenv;
extern crate futures;
extern crate serde_derive;

use actix_web::{
    http::Method,
    App,
    middleware::Logger,
};
use actix::{dev::ToEnvelope, Actor, Addr, SyncArbiter, SyncContext, Handler, MailboxError, Message};
use diesel::{
    mysql::MysqlConnection,
    r2d2::{ConnectionManager, Pool},
};
use dotenv::dotenv;
use std::env;
use oxide_auth::{frontends::actix::*, frontends::actix::message::*, code_grant::{frontend::{OAuthError, OwnerAuthorization}}, primitives::prelude::*};
use std::sync::Arc;
use futures::Future;

pub mod answers;
pub mod index;
pub mod models;
pub mod schema;

pub struct DbExecutor(pub Pool<ConnectionManager<MysqlConnection>>);

impl Actor for DbExecutor {
    type Context = SyncContext<Self>;
}

pub struct AppState<create_authorization, create_grant, create_access> {
    pub db: Addr<DbExecutor>,
    pub endpoint: Addr<CodeGrantEndpoint<create_authorization, create_grant, create_access>>,
}

fn init_oauth_clients() -> ClientMap {
    let mut clients = ClientMap::new();
    let client = Client::public("postman", "https://www.getpostman.com/oauth2/callback".parse().unwrap(), "default".parse().unwrap());
    clients.register_client(client);
    clients
}

pub fn create_authorization(client: &ClientMap, authorizer: &Storage<RandomGenerator>, issuer: &TokenSigner, scopes: &'static[Scope]) -> AuthorizationFlow {
//    AuthorizationFlow::new(client, authorizer)
}

pub fn create_grant(client: &ClientMap, authorizer: &Storage<RandomGenerator>, issuer: &TokenSigner, scopes: &'static[Scope]) -> GrantFlow {
//    GrantFlow::new(client, authorizer, issuer)
}

pub fn create_access(client: &ClientMap, authorizer: &Storage<RandomGenerator>, issuer: &TokenSigner, scopes: &'static[Scope]) -> AccessFlow {
//    AccessFlow::new(issuer, scopes)
}

pub fn create_app() -> App<AppState<create_authorization, create_grant, create_access>> {
    dotenv().ok();
    env_logger::init();

    let database_url = env::var("DATABASE_URL").expect("DATABASE_URL must be set.");
    let manager = ConnectionManager::<MysqlConnection>::new(database_url);

    let pool = Pool::builder()
        .build(manager)
        .expect("Failed to create pool.");
    let authorizer = Storage::new(RandomGenerator::new(16));
    let issuer = TokenSigner::ephemeral();
    let scopes = vec!["default".parse().unwrap()].into_boxed_slice();
    let clients = init_oauth_clients();

    let db_addr = SyncArbiter::start(3, move || DbExecutor(pool.clone()));
    let endpoint_addr = CodeGrantEndpoint::new((clients, authorizer, issuer, scopes))
        .with_authorization(|&mut (ref client, ref mut authorizer, _, _)| {
            AuthorizationFlow::new(client, authorizer)
        })
        .with_grant(|&mut (ref client, ref mut authorizer, ref mut issuer, _)| {
            GrantFlow::new(client, authorizer, issuer)
        })
        .with_guard(move |&mut (_, _, ref mut issuer, ref mut scope)| {
            AccessFlow::new(issuer, scope)
        })
        .start();

    App::with_state(AppState {db: db_addr, endpoint: endpoint_addr})
        .middleware(Logger::default())
        .resource("/", |r| r.method(Method::GET).f(index::get))
        .resource("/answers", |r| r.method(Method::POST).f(answers::post))
}
