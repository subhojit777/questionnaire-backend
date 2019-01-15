use crate::AppState;
use actix_web::HttpRequest;

pub fn get(_req: &HttpRequest<AppState>) -> String {
    String::from("Welcome to Questionnaire!")
}
