use serde_derive::*;

#[derive(Queryable)]
pub struct Question {
    pub id: i32,
    pub title: String,
    pub created: i32,
}

#[derive(Deserialize, Queryable, Serialize, Debug)]
pub struct Answer {
    pub id: i32,
    pub question_id: i32,
    pub title: String,
    pub user_id: i32,
    pub created: i32,
}
