use crate::models::email::{Email, NewEmail};
use crate::schema::{emails, users};
use crate::sql_types::Role;
use bcrypt::{hash as bcrypt_hash, verify as bcrypt_verify, BcryptError, DEFAULT_COST};
use chrono::NaiveDateTime;
use diesel::prelude::*;
use diesel::{self, insert_into};
use serde_derive::{Deserialize, Serialize};
use std::error;
use std::fmt;

#[derive(Debug)]
pub enum AuthenticationError {
    IncorrectPassword,
    NoUsernameSet,
    NoPasswordSet,
    BcryptError(BcryptError),
    DatabaseError(diesel::result::Error),
}

impl fmt::Display for AuthenticationError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(
            f,
            "authentication error: {}",
            match *self {
                AuthenticationError::IncorrectPassword => "incorrect password",
                AuthenticationError::NoUsernameSet => "no username set",
                AuthenticationError::NoPasswordSet => "no password set",
                _ => "internal error",
            }
        )
    }
}

// This is important for other errors to wrap this one.
impl error::Error for AuthenticationError {
    fn source(&self) -> Option<&(dyn error::Error + 'static)> {
        // Generic error, underlying cause isn't tracked.
        None
    }
}

impl From<BcryptError> for AuthenticationError {
    fn from(e: BcryptError) -> Self {
        AuthenticationError::BcryptError(e)
    }
}

impl From<diesel::result::Error> for AuthenticationError {
    fn from(e: diesel::result::Error) -> Self {
        AuthenticationError::DatabaseError(e)
    }
}

pub use self::AuthenticationError::{IncorrectPassword, NoPasswordSet, NoUsernameSet};

#[derive(
    Deserialize,
    Serialize,
    Clone,
    Debug,
    PartialEq,
    Eq,
    Queryable,
    Identifiable,
    AsChangeset,
    Associations,
)]
pub struct User {
    pub id: i32,
    pub role: Role,
    pub username: String,
    pub profile_name: String,
    pub profile_image: String,
    pub last_sign_in_at: Option<NaiveDateTime>,
    pub joined_at: NaiveDateTime,
    pub updated_at: NaiveDateTime,
}

// user with credential
#[derive(Queryable)]
pub struct UserWithPassword {
    user: User,
    password: String,
}

pub fn find_user(conn: &PgConnection, id: i32) -> Result<Option<User>, AuthenticationError> {
    users::table
        .filter(users::id.eq(id))
        .select((
            users::id,
            users::role,
            users::username,
            users::profile_name,
            users::profile_image,
            users::last_sign_in_at,
            users::joined_at,
            users::updated_at,
        ))
        .first::<User>(&*conn)
        .optional()
        .map_err(AuthenticationError::DatabaseError)
}

pub fn try_user_login(
    conn: &PgConnection,
    username: &str,
    password: &str,
) -> Result<Option<User>, AuthenticationError> {
    let user_and_password = users::table
        .filter(users::username.eq(username))
        .select((
            (
                users::id,
                users::role,
                users::username,
                users::profile_name,
                users::profile_image,
                users::last_sign_in_at,
                users::joined_at,
                users::updated_at,
            ),
            users::hashed_password,
        ))
        .first::<UserWithPassword>(&*conn)
        .optional()
        .map_err(AuthenticationError::DatabaseError)?;

    if let Some(user_and_password) = user_and_password {
        if bcrypt_verify(password, &user_and_password.password)? {
            diesel::update(users::table.find(user_and_password.user.id))
                .set(users::last_sign_in_at.eq(diesel::expression::dsl::now))
                .execute(conn)?;
            Ok(Some(user_and_password.user))
        } else {
            Err(IncorrectPassword)
        }
    } else {
        // run hashed here so it take times like existing username
        let _ = bcrypt_hash(password, DEFAULT_COST)?;

        Ok(None)
    }
}

pub fn register_user(
    conn: &PgConnection,
    username: &str,
    email: &str,
    password: &str,
    role: &Role,
) -> Result<User, AuthenticationError> {
    let hashed_password = bcrypt_hash(password, DEFAULT_COST)?;

    conn.transaction(|| {
        let user = insert_into(users::table)
            .values((
                users::username.eq(username),
                users::role.eq(role),
                users::hashed_password.eq(hashed_password),
                users::profile_name.eq(""),
                users::profile_image.eq(""),
            ))
            .returning((
                users::id,
                users::role,
                users::username,
                users::profile_name,
                users::profile_image,
                users::last_sign_in_at,
                users::joined_at,
                users::updated_at,
            ))
            .get_result::<User>(&*conn)
            .map_err(AuthenticationError::DatabaseError)?;

        let new_email = NewEmail {
            email: email,
            user_id: user.id,
            is_primary: true,
        };

        let token = insert_into(emails::table)
            .values(&new_email)
            .on_conflict_do_nothing()
            .returning(emails::token)
            .get_result::<String>(&*conn)
            .optional()?;

        if let Some(token) = token {
            crate::email::send_user_confirm_email(email, username, &token);
        }

        Ok(user)
    })
}

pub fn regenerate_email_token_and_send(
    conn: &PgConnection,
    user_id: i32,
) -> Result<bool, AuthenticationError> {
    use diesel::dsl::sql;
    use diesel::update;

    conn.transaction(|| {
        let user = find_user(conn, user_id)?;

        if let Some(user) = user {
            let email = update(Email::belonging_to(&user))
                .set(emails::token.eq(sql("DEFAULT")))
                .get_result::<Email>(&*conn)
                .map_err(AuthenticationError::DatabaseError)?;

            crate::email::send_user_confirm_email(&email.email, &user.username, &email.token);

            Ok(true)
        } else {
            Ok(false)
        }
    })
}

/// verify an email address based on token
pub fn verify_email_with_token(
    conn: &PgConnection,
    token: &str,
) -> Result<bool, AuthenticationError> {
    use diesel::update;

    let updated_rows = update(emails::table.filter(emails::token.eq(token)))
        .set(emails::verified.eq(true))
        .execute(&*conn)
        .map_err(AuthenticationError::DatabaseError)?;

    Ok(updated_rows > 0)
}

// update a user
#[derive(AsChangeset)]
#[table_name = "users"]
pub struct UserChanges {
    pub role: Option<Role>,
    pub profile_name: Option<String>,
    pub profile_image: Option<String>,
}

// update a user
pub fn update_user(
    conn: &PgConnection,
    user_id: i32,
    user: &UserChanges,
) -> Result<User, AuthenticationError> {
    use crate::schema::users::dsl::*;
    use diesel::update;

    let user = update(users.find(user_id))
        .set(user)
        .returning((
            id,
            role,
            username,
            profile_name,
            profile_image,
            last_sign_in_at,
            joined_at,
            updated_at,
        ))
        .get_result::<User>(&*conn)
        .map_err(AuthenticationError::DatabaseError)?;

    Ok(user)
}

pub fn user_verified_email(
    conn: &PgConnection,
    user_id: i32,
) -> Result<Option<String>, AuthenticationError> {
    emails::table
        .select(emails::email)
        .filter(emails::user_id.eq(user_id))
        .filter(emails::verified.eq(true))
        .first(&*conn)
        .optional()
        .map_err(AuthenticationError::DatabaseError)
}

pub fn user_email(
    conn: &PgConnection,
    user_id: i32,
) -> Result<Option<String>, AuthenticationError> {
    emails::table
        .select(emails::email)
        .filter(emails::user_id.eq(user_id))
        .first(&*conn)
        .optional()
        .map_err(AuthenticationError::DatabaseError)
}
