use std::error;
use std::fmt;

use bcrypt::{DEFAULT_COST, hash as bcrypt_hash, BcryptError};
use diesel::prelude::*;
use diesel::{self, insert_into};
use crate::models::email::{NewEmail};
use crate::schema::{emails, users};

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
        write!(f, "authentication error")
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

#[derive(Clone, Debug, PartialEq, Eq, Queryable, Identifiable, AsChangeset, Associations)]
pub struct User {
    pub id: i32,
    pub username: String,
    pub profile_name: String,
    pub profile_image: String,
}

pub fn register_user(
    conn: &PgConnection,
    username: &str,
    email: &str,
    password: &str,
) -> Result<User, AuthenticationError> {
    let hashed_password = bcrypt_hash(password, DEFAULT_COST)?;

    conn.transaction(|| {
        let user = insert_into(users::table)
            .values((
                users::username.eq(username),
                users::hashed_password.eq(hashed_password),
                users::profile_name.eq(""),
                users::profile_image.eq(""),
            ))
            .returning((users::id, users::username, users::profile_name, users::profile_image))
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