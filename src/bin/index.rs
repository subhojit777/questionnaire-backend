use actix_cors::Cors;

use actix_web::middleware::Logger;
use actix_web::App;
use actix_web::HttpServer;
use diesel::r2d2::{ConnectionManager, Pool};
use diesel::MysqlConnection;
use dotenv::dotenv;
use questionnaire_rs::*;

use actix_identity::{CookieIdentityPolicy, IdentityService};
use std::env;

#[actix_rt::main]
async fn main() -> std::io::Result<()> {
    dotenv().ok();
    env_logger::init();
    let server_address = env::var("ADDRESS").expect("Server ADDRESS must be set.");

    HttpServer::new(|| {
        let front_end_base_url = env::var("FRONT_END_BASE_URL").unwrap_or(String::from(""));
        let database_url = env::var("DATABASE_URL").expect("DATABASE_URL must be set.");
        let max_database_pool_size: u32 = env::var("MAX_DATABASE_POOL_SIZE")
            .unwrap()
            .as_str()
            .parse::<u32>()
            .unwrap();
        let manager = ConnectionManager::<MysqlConnection>::new(database_url);

        let pool = Pool::builder()
            .max_size(max_database_pool_size)
            .build(manager)
            .expect("Failed to create pool.");
        dbg!(pool.max_size());
        dbg!("show max_size");

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
                    .allowed_origin(&front_end_base_url)
                    .supports_credentials()
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
            .service(session::is_logged_in)
            .service(web_socket::index)
    })
    .bind(server_address)
    .unwrap()
    .run()
    .await
}
