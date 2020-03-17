use actix_cors::Cors;

use actix_web::http::{header, Method};
use actix_web::middleware::Logger;
use actix_web::App;
use actix_web::HttpServer;
use diesel::r2d2::{ConnectionManager, Pool};
use diesel::MysqlConnection;
use dotenv::dotenv;

use actix_identity::{CookieIdentityPolicy, IdentityService};
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
            .wrap(IdentityService::new(
                CookieIdentityPolicy::new(&[0; 32])
                    .name("auth-cookie")
                    .secure(false),
            ))
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
            .service(answers::get_by_option)
            .service(options::post)
            .service(options::get)
            .service(options::get_by_question)
            .service(presentations::post)
            .service(presentations::get)
            .service(questions::get)
            .service(questions::post)
            .service(questions::get_by_presentation)
            .service(session::login)
            .service(session::logout)
    })
    .bind("127.0.0.1:8088")
    .unwrap()
    .run()
    .await
}
