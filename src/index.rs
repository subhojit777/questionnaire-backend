extern crate actix_web;
extern crate diesel;

use actix_web::HttpRequest;
use diesel::prelude::*;
use models::Question;
use std::fmt::Write;

pub fn get(_req: &HttpRequest) -> String {
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
