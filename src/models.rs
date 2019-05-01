use chrono::NaiveDateTime;
use diesel::Insertable;
use diesel::Queryable;
use schema::answers;
use serde_derive::*;

#[derive(Queryable)]
pub struct Questions {
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

#[derive(Insertable, Debug)]
#[table_name = "answers"]
pub struct NewAnswer {
    pub question_id: i32,
    pub title: String,
    pub user_id: i32,
    pub created: NaiveDateTime,
}

impl NewAnswer {
    pub fn new(question_id: i32, title: String, user_id: i32, created: NaiveDateTime) -> Self {
        NewAnswer {
            question_id,
            title,
            user_id,
            created,
        }
    }
}

#[derive(Deserialize, Serialize, Debug)]
pub struct AnswerInput {
    pub question_id: i32,
    pub title: String,
}

#[derive(Serialize, Deserialize)]
pub struct GHAccessTokenBody {
    client_id: String,
    client_secret: String,
    code: String,
    accept: String,
}

impl GHAccessTokenBody {
    pub fn new(client_id: String, client_secret: String, code: String, accept: String) -> Self {
        GHAccessTokenBody {
            client_id,
            client_secret,
            code,
            accept,
        }
    }
}

#[derive(Queryable, Deserialize)]
pub struct AnswerId(pub i32);
