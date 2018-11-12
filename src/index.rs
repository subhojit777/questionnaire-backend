use actix_web::HttpRequest;
use crate::AppState;

pub fn get(_req: &HttpRequest<AppState>) -> String {
    String::from("Welcome to Questionnaire!")
}
