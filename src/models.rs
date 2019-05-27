use chrono::NaiveDateTime;
use diesel::Insertable;
use diesel::Queryable;
use schema::answers;
use schema::presentations;
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

/// Creates a new answer.
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

/// The structure of the body of JSON request for creating a new answer.
///
/// This is used for making the API request, and `NewAnswer` is used by the application for creating
/// the answer in database.
#[derive(Deserialize, Serialize, Debug)]
pub struct AnswerInput {
    pub question_id: i32,
    pub title: String,
}

/// The structure of the body of JSON request for retrieving the access token for a session code.
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

/// This defines an actor for retrieving answer from database by id.
#[derive(Queryable, Deserialize)]
pub struct GetAnswerById(pub i32);

/// Presentation model.
#[derive(Queryable, Serialize, Deserialize)]
pub struct Presentation {
    id: i32,
    title: String,
    user_id: i32,
    created: NaiveDateTime,
}

/// Creates a new presentation.
#[derive(Insertable, Debug)]
#[table_name = "presentations"]
pub struct NewPresentation {
    title: String,
    user_id: i32,
    created: NaiveDateTime,
}

/// The structure of the body of JSON request for creating a new presentation.
///
/// This is used for making the API request, and `NewPresentation` is used by the application for
/// creating the presentation in database.
#[derive(Deserialize, Serialize, Debug)]
pub struct PresentationInput {
    pub title: String,
}

impl NewPresentation {
    pub fn new(title: String, user_id: i32, created: NaiveDateTime) -> Self {
        NewPresentation {
            title,
            user_id,
            created,
        }
    }
}
