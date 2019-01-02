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
    HttpRequest,
    HttpResponse,
    Error as AWError
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

#[derive(Clone)]
pub struct AppState {
    pub db: Addr<DbExecutor>,
    pub endpoint: Addr<CodeGrantEndpoint<
        State,
        fn(&mut State) -> AuthorizationFlow,
        fn(&mut State) -> GrantFlow,
        fn(&mut State) -> AccessFlow>>,
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
    fn(&mut State) -> AccessFlow>;

fn init_oauth_clients() -> ClientMap {
    let mut clients = ClientMap::new();
    let client = Client::public("postman", "https://www.getpostman.com/oauth2/callback".parse().unwrap(), "default".parse().unwrap());
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
    let endpoint_addr: Addr<FnEndpoint<State>> = CodeGrantEndpoint::<State>
        ::new(state)
        .with_authorization::<fn(&mut State) -> AuthorizationFlow>(endpoint_authorization)
        .with_grant::<fn(&mut State) -> GrantFlow>(endpoint_grant)
        .with_guard::<fn(&mut State) -> AccessFlow>(endpoint_guard)
        .start();

    App::with_state(AppState {db: db_addr, endpoint: endpoint_addr})
        .middleware(Logger::default())
        .resource("/", |r| r.method(Method::GET).f(index::get))
        .resource("/answers", |r| r.method(Method::POST).f(|req: &HttpRequest<AppState>| {
            let state: AppState = req.state().clone();
            Box::new(req.oauth2()
                .guard()
                .and_then(move |request| state.endpoint.send(request)
                    .map_err(|_| OAuthError::InvalidRequest)
                    .and_then(|result| result)
                )
                .map(|()|
                    HttpResponse::Ok()
                        .content_type("text/plain")
                        .body("this should create new answer"))
                .or_else(|error| {
                    Ok(ResolvedResponse::response_or_error((error))
                        .actix_response()
                        .into_builder()
                        .content_type("text/plain")
                        .body("something wrong happened"))
                })) as Box<Future<Item = HttpResponse, Error = AWError>>
        }))
}
