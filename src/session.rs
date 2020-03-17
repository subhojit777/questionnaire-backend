use crate::models::User;
use actix_identity::Identity;
use actix_session::Session;
use actix_web::post;
use actix_web::web::Json;
use actix_web::{get, HttpResponse};
use chrono::Utc;
use diesel::prelude::*;
use diesel::result::Error as DieselError;
use diesel::MysqlConnection;
use serde::Deserialize;

#[derive(Deserialize)]
pub struct UserLogin {
    name: String,
}

/// Logs out the user.
///
/// `/logout` GET
///
/// Headers:
///
/// ```txt
/// Authorization: token <access_token>
/// ```
///
/// Response: 200 OK
#[get("/logout")]
pub fn logout(session: Session) -> HttpResponse {
    session.clear();
    HttpResponse::Ok().finish()
}

/// Logs in a user.
///
/// `/login` POST
///
/// Headers:
///
/// Content type: application/json
///
/// Body:
///
/// ```json
/// {
///    "name": "agent 42",
/// }
/// ```
///
/// Response: 200 OK
#[post("/login")]
pub fn login(data: Json<UserLogin>, id: Identity) -> HttpResponse {
    let input = data.into_inner();
    id.remember(input.name);
    HttpResponse::Ok().finish()
}

pub fn get_user_by_name(name: String, connection: &MysqlConnection) -> Result<User, DieselError> {
    use crate::schema::users::dsl::{name as user_name, users};

    // TODO: This is creating new user everytime.
    let result = users
        .filter(user_name.eq(name.clone()))
        .first(connection)
        .unwrap_or(create_user(name.clone(), &connection)?);

    Ok(result)
}

pub fn create_user(name: String, connection: &MysqlConnection) -> Result<User, DieselError> {
    use crate::schema::users::dsl::{created, name as user_name, users};

    let now = Utc::now().naive_utc();

    diesel::insert_into(users)
        .values((user_name.eq(&name), created.eq(now)))
        .execute(connection)
        .expect("Error creating new user.");

    let new_user = users
        .filter(user_name.eq(&name))
        .first(connection)
        .expect("Error retrieving the new user.");

    Ok(new_user)
}
