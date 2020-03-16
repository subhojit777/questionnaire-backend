use crate::schema::answers;
use crate::schema::options;
use crate::schema::presentations;
use crate::schema::questions;
use crate::schema::users;
use chrono::NaiveDateTime;
use diesel::Insertable;
use diesel::Queryable;
use serde_derive::*;

#[derive(Queryable, Serialize, Deserialize, Identifiable, Associations)]
#[belongs_to(Presentation, foreign_key = "presentation_id")]
#[table_name = "questions"]
pub struct Questions {
    pub id: i32,
    pub title: String,
    pub created: NaiveDateTime,
    pub presentation_id: i32,
    pub user_id: i32,
}

/// Creates a new question.
#[derive(Insertable, Debug)]
#[table_name = "questions"]
pub struct NewQuestion {
    pub title: String,
    pub created: NaiveDateTime,
    pub presentation_id: i32,
    pub user_id: i32,
}

impl NewQuestion {
    pub fn new(title: String, created: NaiveDateTime, presentation_id: i32, user_id: i32) -> Self {
        NewQuestion {
            title,
            created,
            presentation_id,
            user_id,
        }
    }
}

/// The structure of the body of JSON request for creating a new question.
///
/// This is used for making the API request, and `NewQuestion` is used by the application for
/// creating the question in database.
#[derive(Deserialize, Serialize, Debug)]
pub struct NewQuestionJson {
    pub title: String,
    pub presentation_id: i32,
}

/// This defines an actor for retrieving question from database by id.
#[derive(Queryable, Deserialize)]
pub struct GetQuestion(pub i32);

/// Defines an actor for retrieving questions for a presentation.
#[derive(Queryable, Deserialize, Associations)]
#[belongs_to(Presentation, foreign_key = "presentation_id")]
#[table_name = "questions"]
pub struct GetQuestionByPresentation {
    pub presentation_id: i32,
}

#[derive(Queryable, Serialize, Deserialize, Identifiable, Associations)]
#[belongs_to(Option, foreign_key = "option_id")]
#[table_name = "answers"]
pub struct Answer {
    pub id: i32,
    pub user_id: i32,
    pub created: NaiveDateTime,
    pub option_id: i32,
}

/// Creates a new answer.
#[derive(Insertable, Debug, Associations)]
#[belongs_to(Option, foreign_key = "option_id")]
#[table_name = "answers"]
pub struct NewAnswer {
    pub user_id: i32,
    pub created: NaiveDateTime,
    pub option_id: i32,
}

impl NewAnswer {
    pub fn new(user_id: i32, created: NaiveDateTime, option_id: i32) -> Self {
        NewAnswer {
            user_id,
            created,
            option_id,
        }
    }
}

/// The structure of the body of JSON request for creating a new answer.
///
/// This is used for making the API request, and `NewAnswer` is used by the application for creating
/// the answer in database.
#[derive(Deserialize, Serialize, Debug)]
pub struct AnswerInput {
    pub option_id: i32,
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
#[derive(Queryable, Serialize, Deserialize, Identifiable)]
#[table_name = "presentations"]
pub struct Presentation {
    pub id: i32,
    pub title: String,
    pub user_id: i32,
    pub created: NaiveDateTime,
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

/// Defines an actor to retrieve presentation from database by id.
#[derive(Queryable, Deserialize)]
pub struct GetPresentation(pub i32);

/// Option model.
#[derive(Queryable, Serialize, Deserialize, Identifiable, Associations)]
#[belongs_to(Questions, foreign_key = "question_id")]
#[table_name = "options"]
pub struct Option {
    pub id: i32,
    pub data: String,
    pub user_id: i32,
    pub question_id: i32,
    pub created: NaiveDateTime,
}

/// Creates a new option.
#[derive(Insertable, Debug)]
#[table_name = "options"]
pub struct NewOption {
    pub data: String,
    pub user_id: i32,
    pub question_id: i32,
    pub created: NaiveDateTime,
}

impl NewOption {
    pub fn new(data: String, user_id: i32, question_id: i32, created: NaiveDateTime) -> Self {
        NewOption {
            data,
            user_id,
            question_id,
            created,
        }
    }
}

/// Defines the structure of the body of JSON request for creating a new option.
///
/// This is used for making the API request, and `NewOption` is used by the application for
/// creating the option in database.
#[derive(Deserialize, Serialize, Debug)]
pub struct NewOptionJson {
    pub data: String,
    pub question_id: i32,
}

/// Defines an actor to retrieve an option from database by id.
#[derive(Queryable, Deserialize)]
pub struct GetOption(pub i32);

/// Defines an actor for retrieving options for a question.
#[derive(Queryable, Deserialize, Associations)]
#[belongs_to(Questions, foreign_key = "question_id")]
#[table_name = "options"]
pub struct GetOptionsByQuestion {
    pub question_id: i32,
}

/// Defines an actor for retrieving answers for a question.
#[derive(Queryable, Deserialize, Associations)]
#[belongs_to(Questions, foreign_key = "option_id")]
#[table_name = "answers"]
pub struct GetAnswersByOption {
    pub option_id: i32,
}

#[derive(Queryable, Serialize, Deserialize, Identifiable)]
#[table_name = "users"]
pub struct User {
    pub id: i32,
    pub name: String,
    pub created: NaiveDateTime,
}
