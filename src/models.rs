use serde_derive::*;
use chrono::NaiveDateTime;
use schema::answers;
use diesel::Queryable;
use diesel::Insertable;

#[derive(Queryable)]
pub struct Question {
    pub id: i32,
    pub title: String,
    pub created: NaiveDateTime,
}

#[derive(Queryable, Serialize)]
pub struct Answer {
    pub id: i32,
    pub question_id: i32,
    pub title: String,
    pub user_id: i32,
    pub created: NaiveDateTime,
}

#[derive(Deserialize, Insertable)]
#[table_name = "answers"]
pub struct AnswerForm {
    pub question_id: i32,
    pub title: String,
    pub user_id: i32,
}
