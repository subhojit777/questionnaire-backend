use crate::models::User;
use crate::DbPool;
use actix_identity::Identity;
use actix_web::post;
use actix_web::web::{block, Data, Json};
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
/// Response: 200 OK
#[get("/logout")]
pub fn logout(id: Identity) -> HttpResponse {
    id.forget();
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
pub async fn login(
    data: Json<UserLogin>,
    pool: Data<DbPool>,
    id: Identity,
) -> Result<HttpResponse, actix_web::Error> {
    let input = data.into_inner();
    let connection = pool.get().expect("Could not get database connection");
    let user_name = input.name.clone();

    block(move || get_user_by_name(user_name, &connection))
        .await
        .map_err(|_| HttpResponse::InternalServerError().body("Something went wrong during login."))
        .expect("Could not locate user by name during login.");

    id.remember(input.name);
    Ok(HttpResponse::Ok().finish())
}

pub fn get_user_by_name(name: String, connection: &MysqlConnection) -> Result<User, DieselError> {
    use crate::schema::users::dsl::{name as user_name, users};

    let result = users.filter(user_name.eq(name.clone())).first(connection);

    if result.is_err() {
        return create_user(name.clone(), &connection);
    }

    Ok(result.expect("Could not locate user by name."))
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
