use std::error;
use std::fmt;

use bcrypt::{DEFAULT_COST, hash as bcrypt_hash, verify as bcrypt_verify, BcryptError};
use diesel::prelude::*;
use diesel::{self, insert_into};
use serde_derive::{Deserialize, Serialize};
use crate::models::email::{NewEmail};
use crate::schema::{emails, users};
use crate::sql_types::Role;

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
                AuthenticationError::NoUsernameSet     => "no username set",
                AuthenticationError::NoPasswordSet     => "no password set",
                _                                      => "internal error",
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

#[derive(Deserialize, Serialize, Clone, Debug, PartialEq, Eq, Queryable, Identifiable, AsChangeset, Associations)]
pub struct User {
    pub id: i32,
    pub role: Role,
    pub username: String,
    pub profile_name: String,
    pub profile_image: String,
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
        .select(
            (users::id, users::role, users::username, users::profile_name, users::profile_image),
        )
        .first::<User>(conn)
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
        .select(
            (
                (users::id, users::role, users::username, users::profile_name, users::profile_image),
                users::hashed_password,
            ),
        )
        .first::<UserWithPassword>(conn)
        .optional()
        .map_err(AuthenticationError::DatabaseError)?;

    if let Some(user_and_password) = user_and_password {
        if bcrypt_verify(password, &user_and_password.password)? {
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
            .returning((users::id, users::role, users::username, users::profile_name, users::profile_image))
            .get_result::<User>(conn)
            .map_err(AuthenticationError::DatabaseError)?;
        
        let new_email = NewEmail {
            email: email,
            user_id: user.id,
        };

        let token = insert_into(emails::table)
            .values(&new_email)
            .on_conflict_do_nothing()
            .returning(emails::token)
            .get_result::<String>(conn)
            .optional()?;
        
        if let Some(token) = token {
            crate::email::send_user_confirm_email(email, username, &token);
        }

        Ok(user)
    })
}

/// verify an email address based on token
pub fn verify_email_with_token(conn: &PgConnection, token: &str) -> Result<bool, AuthenticationError> {
    use diesel::update;

    let updated_rows = update(emails::table.filter(emails::token.eq(token)))
        .set(emails::verified.eq(true))
        .execute(conn)
        .map_err(AuthenticationError::DatabaseError)?;

    Ok(updated_rows > 0)
}
