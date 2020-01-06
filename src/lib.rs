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
//!   "option_id": 23
//! }
//! ```
//!
//! **Response:** 200 OK
//!
//! #### `/answers/{id}`
//!
//! **Method:** GET
//!
//! **Response:**
//!
//! ```json
//! {
//!    "id": 47,
//!    "user_id": 7,
//!    "created": "2019-11-01T14:30:30",
//!    "option_id": 23
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
//! #### `/questions`
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
//!
//! #### Get questions for a presentation.
//!
//! **Endpoint:** `/questions-presentation`
//!
//! **Parameters:**
//!
//! ```txt
//! presentation_id: <id>
//! ```
//!
//! **Method:** GET
//!
//! **Response:**
//!
//! ```json
//! [
//!    {
//!         "id": 23,
//!         "title": "New Question",
//!         "created": "2019-11-01T14:30:30",
//!         "presentation_id": 3,
//!         "user_id": 7,
//!     }
//! ]
//! ```
//!
//! #### `/options`
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
//!    "data": "Option 1",
//!    "question_id": 1,
//! }
//! ```
//!
//! **Response:** 200 OK
//!
//! #### `/options/{id}`
//!
//! **Method:** GET
//!
//! **Response:**
//!
//! ```json
//! {
//!    "id": 12,
//!    "data": "Option 1",
//!    "user_id": 9,
//!    "question_id": 1,
//!    "created": "2019-06-19T03:40:50"
//! }
//! ```
//!
//! #### Get options for a question
//!
//! **Endpoint:** `/options-question`
//!
//! **Parameters:**
//!
//! ```txt
//! question_id: <id>
//! ```
//!
//! **Method:** GET
//!
//! **Response:**
//!
//! ```json
//! [
//!    {
//!         "id": 12,
//!         "data": "Option 1",
//!         "user_id": 9,
//!         "question_id": 1,
//!         "created": "2019-06-19T03:40:50"
//!     }
//! ]
//! ```
//!
//! #### `/gh-access-token`
//!
//! **Method:** GET
//!
//! **Parameters:**
//!
//! ```txt
//! code: GITHUB_LOGIN_CODE obtained from https://github.com/login/oauth/authorize
//! ```
//!
//! **Response:**
//!
//! GITHUB_ACCESS_TOKEN in JSON.
//!
//! #### Get answers for an option
//!
//! **Endpoint:** `/answers-option`
//!
//! **Parameters:**
//!
//! ```txt
//! option_id: <id>
//! ```
//!
//! **Method:** GET
//!
//! **Response:**
//!
//! ```json
//! [
//!    {
//!         "id": 12,
//!         "user_id": 9,
//!         "created": "2019-06-19T03:40:50",
//!         "option_id": 1,
//!     },
//!    {
//!         "id": 13,
//!         "user_id": 18,
//!         "created": "2019-06-30T03:40:50",
//!         "option_id": 3,
//!     }
//! ]
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
extern crate time;

use actix_web::middleware::cors::Cors;
use actix_web::middleware::session::{CookieSessionBackend, SessionStorage};
use actix_web::{
    actix::{Actor, Addr, SyncArbiter, SyncContext},
    http::{header, Method},
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
use time::Duration;

pub mod answers;
pub mod error;
pub mod github_access_token;
pub mod helpers;
pub mod middleware;
pub mod models;
pub mod options;
pub mod presentations;
pub mod questions;
pub mod schema;
pub mod session;

const GH_USER_SESSION_ID_KEY: &str = "gh_user_id";
const SAFE_PATHS: [&str; 7] = [
    "/gh-access-token",
    "/answers/{id}",
    "/presentations/{id}",
    "/questions/{id}",
    "/questions-presentation",
    "/options/{id}",
    "/options-question",
];

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

    let front_end_base_url = env::var("FRONT_END_BASE_URL").unwrap_or(String::from(""));
    let database_url = env::var("DATABASE_URL").expect("DATABASE_URL must be set.");
    let manager = ConnectionManager::<MysqlConnection>::new(database_url);

    let pool = Pool::builder()
        .build(manager)
        .expect("Failed to create pool.");

    let addr = SyncArbiter::start(3, move || DbExecutor(pool.clone()));

    App::with_state(AppState { db: addr.clone() })
        .middleware(Logger::default())
        .middleware(SessionStorage::new(
            CookieSessionBackend::signed(&[0; 32])
                .secure(false)
                .max_age(Duration::days(1)),
        ))
        .middleware(GitHubUserId::default())
        .middleware(
            Cors::build()
                .allowed_headers(vec![
                    header::AUTHORIZATION,
                    header::ACCEPT,
                    header::CONTENT_TYPE,
                ])
                .allowed_methods(vec![Method::GET, Method::POST])
                .allowed_origin(&front_end_base_url)
                .finish(),
        )
        .resource("/answers", |r| {
            r.method(Method::POST).with_async(answers::post)
        })
        .resource("/answers/{id}", |r| {
            r.method(Method::GET).with_async(answers::get)
        })
        .resource("/logout", |r| r.method(Method::GET).f(session::logout))
        .resource("/presentations", |r| {
            r.method(Method::POST).with_async(presentations::post)
        })
        .resource("/presentations/{id}", |r| {
            r.method(Method::GET).with_async(presentations::get)
        })
        .resource("/questions", |r| {
            r.method(Method::POST).with_async(questions::post)
        })
        .resource("/questions/{id}", |r| {
            r.method(Method::GET).with_async(questions::get)
        })
        .resource("/questions-presentation", |r| {
            r.method(Method::GET)
                .with_async(questions::get_by_presentation)
        })
        .resource("/options", |r| {
            r.method(Method::POST).with_async(options::post)
        })
        .resource("/options/{id}", |r| {
            r.method(Method::GET).with_async(options::get)
        })
        .resource("/options-question", |r| {
            r.method(Method::GET).with_async(options::get_by_question)
        })
        .resource("/gh-access-token", |r| {
            r.method(Method::GET)
                .with_async(github_access_token::get_access_token)
        })
}
