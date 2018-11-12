use actix_web::HttpRequest;
use diesel::prelude::*;
use diesel::r2d2::{ConnectionManager, Pool};
use models::Question;
use std::fmt::Write;
use crate::AppState;

pub struct DbExecutor(pub Pool<ConnectionManager<MysqlConnection>>);

pub fn get(_req: &HttpRequest<AppState>) -> String {
    // TODO: Re-implement this. Check the original app for the home page content.
    use schema::question::dsl::*;

    // TODO: Connection should be established only once. Not per function.
    let connection = ::establish_connection();
    let mut output = String::new();

    let results = question
        .load::<Question>(&connection)
        .expect("Error loading questions");

    for row in results {
        write!(&mut output, "{}\n{}\n", row.title, row.created);
    }

    output
}
