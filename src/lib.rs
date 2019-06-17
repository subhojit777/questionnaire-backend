//! Backend of the Questionnaire app.
//! ### API endpoints available:
//!
//! #### `/answers`
//!
//! **Method:** POST
//!
//! **Headers:**
//!
//! ```txt
//! Content-type: application/json
//! Authorization: token <access_token>
//! ```
//!
//! **Body:**
//!
//! ```json
//! {
//!   "question_id": 23,
//!   "title": "Nothing is as it seems."
//! }
//! ```
//!
//! **Response:** 200 OK
//!
//! #### `/answers/{id}`
//!
//! **Method:** GET
//!
//! **Headers:**
//!
//! ```txt
//! Authorization: token <access_token>
//! ```
//!
//! **Response:**
//!
//! ```json
//! {
//!    "id": 47,
//!    "question_id": 23,
//!    "title": "Nothing is as it seems.",
//!    "user_id": 7,
//!    "created": "2019-11-01T14:30:30"
//! }
//! ```
//!
//! #### `/logout`
//!
//! **Method:** GET
//!
//! **Headers:**
//!
//! ```txt
//! Authorization: token <access_token>
//! ```
//!
//! **Response:** 200 OK
//!
//! #### `/presentations`
//!
//! **Method:** POST
//!
//! **Headers:**
//!
//! ```txt
//! Content-type: application/json
//! Authorization: token <access_token>
//! ```
//!
//! **Body:**
//!
//! ```json
//! {
//!   "title": "New Presentation"
//! }
//! ```
//!
//! **Response:** 200 OK
//!
//! #### `/presentations/{id}`
//!
//! **Method:** GET
//!
//! **Headers:**
//!
//! ```txt
//! Authorization: token <access_token>
//! ```
//!
//! **Response:**
//!
//! ```json
//! {
//!    "id": 47,
//!    "title": "New Presentation",
//!    "user_id": 7,
//!    "created": "2019-11-01T14:30:30"
//! }
//! ```
//!
//! #### ``/questions`
//!
//! **Method:** POST
//!
//! **Headers:**
//!
//! ```txt
//! Content type: application/json
//! Authorization: token <access_token>
//! ```
//!
//! **Body:**
//!
//! ```json
//! {
//!    "title": "New Question",
//!    "presentation_id": 1,
//! }
//! ```
//!
//! **Response:** 200 OK
//!
//! #### `/questions/{id}`
//!
//! **Method:** GET
//!
//! **Headers:**
//!
//! ```txt
//! Authorization: token <access_token>
//! ```
//!
//! **Response:**
//!
//! ```json
//! {
//!    "id": 23,
//!    "title": "New Question",
//!    "created": "2019-11-01T14:30:30",
//!    "presentation_id": 3,
//!    "user_id": 7,
//! }
//! ```
extern crate chrono;
extern crate env_logger;
extern crate reqwest;
extern crate serde_json;
#[macro_use]
extern crate diesel;
extern crate actix;
extern crate actix_web;
extern crate dotenv;
extern crate failure;
extern crate futures;
extern crate serde;
extern crate serde_derive;

use actix_web::middleware::session::{CookieSessionBackend, SessionStorage};
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
use middleware::GitHubUserId;
use std::env;

pub mod answers;
pub mod error;
pub mod github;
pub mod helpers;
pub mod middleware;
pub mod models;
pub mod presentations;
pub mod questions;
pub mod schema;
pub mod session;

const GH_USER_SESSION_ID_KEY: &str = "gh_user_id";

/// Database execution actor.
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
        .middleware(SessionStorage::new(
            CookieSessionBackend::signed(&[0; 32]).secure(false),
        ))
        .middleware(GitHubUserId::default())
        .resource("/answers", |r| {
            r.method(Method::POST).with_async(answers::post)
        })
        .resource("/answers/{id}", |r| {
            r.method(Method::GET).with_async(answers::get)
        })
        .resource("/gh-redirect", |r| {
            r.method(Method::GET).a(github::login_redirect)
        })
        .resource("/logout", |r| r.method(Method::GET).f(session::logout))
        .resource("/presentations", |r| {
            r.method(Method::POST).with_async(presentations::post)
        })
        .resource("presentations/{id}", |r| {
            r.method(Method::GET).with_async(presentations::get)
        })
        .resource("/questions", |r| {
            r.method(Method::POST).with_async(questions::post)
        })
        .resource("/questions/{id}", |r| {
            r.method(Method::GET).with_async(questions::get)
        })
}
