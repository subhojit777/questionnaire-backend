extern crate chrono;
extern crate env_logger;
extern crate serde_json;
#[macro_use]
extern crate diesel;
extern crate actix_web;
extern crate dotenv;
extern crate failure;
extern crate futures;
extern crate serde;
extern crate serde_derive;

use actix_web::{
    actix::{Actor, Addr, SyncArbiter, SyncContext},
    http::Method,
    middleware::Logger,
    App,
};
use diesel::{
    mysql::MysqlConnection,
    r2d2::{ConnectionManager, Pool},
};
use dotenv::dotenv;
use std::env;

pub mod answers;
pub mod github;
pub mod index;
pub mod models;
pub mod schema;
pub mod oauth_error;

pub struct DbExecutor(pub Pool<ConnectionManager<MysqlConnection>>);

impl Actor for DbExecutor {
    type Context = SyncContext<Self>;
}

pub struct AppState {
    db: Addr<DbExecutor>,
}

pub fn create_app() -> App<AppState> {
    dotenv().ok();
    env_logger::init();

    let database_url = env::var("DATABASE_URL").expect("DATABASE_URL must be set.");
    let manager = ConnectionManager::<MysqlConnection>::new(database_url);

    let pool = Pool::builder()
        .build(manager)
        .expect("Failed to create pool.");

    let addr = SyncArbiter::start(3, move || DbExecutor(pool.clone()));

    App::with_state(AppState { db: addr.clone() })
        .middleware(Logger::default())
        .resource("/", |r| r.method(Method::GET).f(index::get))
        .resource("/answers", |r| {
            r.method(Method::POST).with_async(answers::post)
        })
        .resource("/answers-get", |r| {
            r.method(Method::GET).with_async(answers::get)
        })
        .resource("/gh-login", |r| r.method(Method::GET).f(github::login_page))
        .resource("/gh-redirect", |r| {
            r.method(Method::GET).a(github::login_redirect)
        })
}
