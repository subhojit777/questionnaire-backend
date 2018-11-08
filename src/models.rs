use chrono::NaiveDateTime;
use diesel::Insertable;
use diesel::Queryable;
use schema::answers;
use serde_derive::*;

#[derive(Queryable)]
pub struct Question {
    pub id: i32,
    pub title: String,
    pub created: NaiveDateTime,
}

#[derive(Queryable, Serialize, Deserialize)]
pub struct Answer {
    pub id: i32,
    pub question_id: i32,
    pub title: String,
    pub user_id: i32,
    pub created: NaiveDateTime,
}

#[derive(Deserialize, Serialize, Insertable)]
#[table_name = "answers"]
pub struct AnswerForm {
    pub question_id: i32,
    pub title: String,
    pub user_id: i32,
}
