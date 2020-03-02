use actix::SyncArbiter;
use actix_cors::Cors;
use actix_service::Service;
use actix_session::{CookieSession, Session, UserSession};
use actix_web::dev::Factory;
use actix_web::http::{header, Method};
use actix_web::middleware::Logger;
use actix_web::{App, HttpResponse};
use actix_web::{HttpRequest, HttpServer};
use chrono::Duration;
use diesel::r2d2::{ConnectionManager, Pool};
use diesel::MysqlConnection;
use dotenv::dotenv;
use questionnaire_rs::middleware::GitHubUserId;
use questionnaire_rs::*;
use std::env;

#[actix_rt::main]
async fn main() -> std::io::Result<()> {
    dotenv().ok();
    env_logger::init();

    // TODO: URL should come from environment variable.
    HttpServer::new(|| {
        let front_end_base_url = env::var("FRONT_END_BASE_URL").unwrap_or(String::from(""));
        let database_url = env::var("DATABASE_URL").expect("DATABASE_URL must be set.");
        let manager = ConnectionManager::<MysqlConnection>::new(database_url);

        let pool = Pool::builder()
            .build(manager)
            .expect("Failed to create pool.");

        App::new()
            .data(pool.clone())
            .wrap(Logger::default())
            .wrap(
                CookieSession::signed(&[0; 32])
                    .secure(false)
                    .max_age(Duration::days(1).num_seconds()),
            )
            .wrap(
                Cors::new()
                    .allowed_headers(vec![
                        header::AUTHORIZATION,
                        header::ACCEPT,
                        header::CONTENT_TYPE,
                    ])
                    .allowed_methods(vec![Method::GET, Method::POST])
                    .allowed_origin(&front_end_base_url)
                    .finish(),
            )
            .service(answers::post)
            .service(answers::get)
        // .resource("/answers/{id}", |r| {
        //     r.method(Method::GET).with_async(answers::get)
        // })
        // .resource("/answers-option", |r| {
        //     r.method(Method::GET).with_async(answers::get_by_option)
        // })
        // .resource("/presentations", |r| {
        //     r.method(Method::POST).with_async(presentations::post)
        // })
        // .resource("/presentations/{id}", |r| {
        //     r.method(Method::GET).with_async(presentations::get)
        // })
        // .resource("/questions", |r| {
        //     r.method(Method::POST).with_async(questions::post)
        // })
        // .resource("/questions/{id}", |r| {
        //     r.method(Method::GET).with_async(questions::get)
        // })
        // .resource("/questions-presentation", |r| {
        //     r.method(Method::GET)
        //         .with_async(questions::get_by_presentation)
        // })
        // .resource("/options", |r| {
        //     r.method(Method::POST).with_async(options::post)
        // })
        // .resource("/options/{id}", |r| {
        //     r.method(Method::GET).with_async(options::get)
        // })
        // .resource("/options-question", |r| {
        //     r.method(Method::GET).with_async(options::get_by_question)
        // })
    })
    .bind("127.0.0.1:8088")
    .unwrap()
    .run()
    .await
}
