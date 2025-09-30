use crate::models::{NewUser, User};
use anyhow::Result;
use argon2::{password_hash::SaltString, Argon2, PasswordHasher};
use diesel::prelude::*;
use jsonwebtoken::{encode, EncodingKey, Header};
use rand::rngs::OsRng;
use serde::Serialize;
use uuid::Uuid;

#[derive(Serialize)]
struct Claims {
    sub: Uuid,
    username: String,
    token_version: i32,
    exp: usize,
}

pub fn hash_password(password: &str) -> std::result::Result<String, argon2::password_hash::Error> {
    let argon2 = Argon2::default();
    let salt = SaltString::generate(&mut OsRng);

    let hash = argon2
        .hash_password(password.as_bytes(), &salt)?
        .to_string();
    Ok(hash)
}

pub fn generate_jwt(user: &User, secret: &str, exp: usize) -> String {
    let claims = Claims {
        sub: user.id,
        username: user.username.clone(),
        token_version: user.token_version,
        exp,
    };
    encode(
        &Header::default(),
        &claims,
        &EncodingKey::from_secret(secret.as_ref()),
    )
    .expect("Failed to encode JWT")
}
pub fn create_user(
    conn: &mut PgConnection,
    new_user: NewUser,
    secret: &str,
) -> Result<(User, String)> {
    use anyhow::anyhow;

    if email_exists(conn, &new_user.email)? {
        return Err(anyhow!("Email already in use"));
    }

    if username_exists(conn, &new_user.username)? {
        return Err(anyhow!("Username already in use"));
    }

    use crate::schema::users::dsl::*;
    let user: User = diesel::insert_into(users)
        .values(&new_user)
        .get_result(conn)?;

    let token = generate_jwt(&user, secret, 3600);
    Ok((user, token))
}

pub fn email_exists(conn: &mut PgConnection, email_check: &str) -> Result<bool> {
    use crate::schema::users::dsl::*;
    let exists = users
        .filter(email.eq(email_check))
        .select(id)
        .first::<Uuid>(conn)
        .optional()?;
    Ok(exists.is_some())
}

pub fn username_exists(conn: &mut PgConnection, username_check: &str) -> Result<bool> {
    use crate::schema::users::dsl::*;
    let exists = users
        .filter(username.eq(username_check))
        .select(id)
        .first::<Uuid>(conn)
        .optional()?;
    Ok(exists.is_some())
}
