extern crate actix;
extern crate chrono;
extern crate env_logger;
extern crate oxide_auth;
extern crate serde_json;
#[macro_use]
extern crate diesel;
extern crate actix_web;
extern crate dotenv;
extern crate futures;
extern crate serde_derive;

use actix::{Actor, Addr, SyncArbiter, SyncContext};
use actix_web::{http::Method, middleware::Logger, App};
use diesel::{
    mysql::MysqlConnection,
    r2d2::{ConnectionManager, Pool},
};
use dotenv::dotenv;
use oxide_auth::{code_grant::frontend::*, frontends::actix::*, primitives::prelude::*};
use std::env;

pub mod answers;
pub mod index;
pub mod models;
pub mod schema;
pub mod oauth;

pub struct DbExecutor(pub Pool<ConnectionManager<MysqlConnection>>);

impl Actor for DbExecutor {
    type Context = SyncContext<Self>;
}

#[derive(Clone)]
pub struct AppState {
    pub db: Addr<DbExecutor>,
    pub endpoint: Addr<
        CodeGrantEndpoint<
            State,
            fn(&mut State) -> AuthorizationFlow,
            fn(&mut State) -> GrantFlow,
            fn(&mut State) -> AccessFlow,
        >,
    >,
}

pub struct State {
    clients: ClientMap,
    authorizer: Storage<RandomGenerator>,
    issuer: TokenSigner,
    scopes: Box<[Scope]>,
}

fn endpoint_authorization(state: &mut State) -> AuthorizationFlow {
    AuthorizationFlow::new(&state.clients, &mut state.authorizer)
}

fn endpoint_grant(state: &mut State) -> GrantFlow {
    GrantFlow::new(&state.clients, &mut state.authorizer, &mut state.issuer)
}

fn endpoint_guard(state: &mut State) -> AccessFlow {
    AccessFlow::new(&mut state.issuer, &state.scopes)
}

type FnEndpoint<State> = CodeGrantEndpoint<
    State,
    fn(&mut State) -> AuthorizationFlow,
    fn(&mut State) -> GrantFlow,
    fn(&mut State) -> AccessFlow,
>;

fn init_oauth_clients() -> ClientMap {
    let mut clients = ClientMap::new();
    let client = Client::public(
        "postman",
        "https://www.getpostman.com/oauth2/callback"
            .parse()
            .unwrap(),
        "default".parse().unwrap(),
    );
    clients.register_client(client);
    clients
}

pub fn create_app() -> App<AppState> {
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

    let state = State {
        clients,
        authorizer,
        issuer,
        scopes,
    };

    let db_addr = SyncArbiter::start(3, move || DbExecutor(pool.clone()));
    let endpoint_addr: Addr<FnEndpoint<State>> = CodeGrantEndpoint::<State>::new(state)
        .with_authorization::<fn(&mut State) -> AuthorizationFlow>(endpoint_authorization)
        .with_grant::<fn(&mut State) -> GrantFlow>(endpoint_grant)
        .with_guard::<fn(&mut State) -> AccessFlow>(endpoint_guard)
        .start();

    App::with_state(AppState {
        db: db_addr,
        endpoint: endpoint_addr,
    })
    .middleware(Logger::default())
    .resource("/", |r| r.method(Method::GET).f(index::get))
    .resource("/redirect", |r| r.method(Method::GET).f(answers::redirect_get))
    .resource("/answers", |r| r.method(Method::POST).a(answers::post))
    .resource("/authorize", |r| r.method(Method::GET).a(oauth::authorize_get))
    .resource("/authorize", |r| r.method(Method::POST).a(oauth::authorize_post))
    .resource("/token", |r| r.method(Method::POST).a(oauth::token_post))
}
