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
//! **Endpoint:** `/answers-option/{option_id}`
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
extern crate actix_cors;
extern crate actix_http;
extern crate actix_session;
extern crate actix_web;
extern crate dotenv;
extern crate failure;
extern crate futures;
extern crate serde;
extern crate serde_derive;
extern crate time;

use diesel::{
    mysql::MysqlConnection,
    r2d2::{ConnectionManager, Pool},
};

pub mod answers;
pub mod error;
pub mod helpers;
pub mod middleware;
pub mod models;
pub mod options;
// pub mod presentations;
// pub mod questions;
pub mod schema;
pub mod session;

pub const GH_USER_SESSION_ID_KEY: &str = "gh_user_id";
const SAFE_PATHS: [&str; 8] = [
    "/gh-access-token",
    "/answers/{id}",
    "/presentations/{id}",
    "/questions/{id}",
    "/questions-presentation",
    "/options/{id}",
    "/options-question",
    "/answers-option",
];

pub type DbPool = Pool<ConnectionManager<MysqlConnection>>;
