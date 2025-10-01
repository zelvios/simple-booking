use crate::models::{NewUser, User, UserBasic};
use anyhow::Result;
use argon2::{password_hash::SaltString, Argon2, PasswordHasher, PasswordVerifier};
use chrono::Utc;
use diesel::prelude::*;
use jsonwebtoken::{encode, EncodingKey, Header};
use rand::rngs::OsRng;
use rand::Rng;
use serde::Serialize;
use std::collections::HashMap;
use std::env;
use uuid::Uuid;

fn get_jwt_expire() -> i64 {
    env::var("JWT_EXPIRE_SECONDS")
        .ok()
        .and_then(|s| s.parse().ok())
        .unwrap_or(3600) // 1 hour
}

#[derive(Serialize)]
struct Claims {
    sub: Uuid,
    username: String,
    token_version: i32,
    exp: i64,
}

pub fn hash_password(password: &str) -> std::result::Result<String, argon2::password_hash::Error> {
    let argon2 = Argon2::default();
    let salt = SaltString::generate(&mut OsRng);

    let hash = argon2
        .hash_password(password.as_bytes(), &salt)?
        .to_string();
    Ok(hash)
}

pub fn generate_jwt(user: &User, secret: &str, expire_seconds: i64) -> String {
    let claims = Claims {
        sub: user.id,
        username: user.username.clone(),
        token_version: user.token_version,
        exp: Utc::now().timestamp() + expire_seconds,
    };
    encode(
        &Header::default(),
        &claims,
        &EncodingKey::from_secret(secret.as_ref()),
    )
    .expect("Failed to encode JWT")
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

    let token = generate_jwt(&user, secret, get_jwt_expire());
    Ok((user, token))
}

pub fn get_users(conn: &mut PgConnection) -> QueryResult<Vec<UserBasic>> {
    use crate::schema::roles::dsl as roles_dsl;
    use crate::schema::users::dsl as users_dsl;
    use crate::schema::users_roles::dsl as ur_dsl;

    let user_rows = users_dsl::users
        .select((
            users_dsl::id,
            users_dsl::username,
            users_dsl::email,
            users_dsl::first_name,
            users_dsl::last_name,
        ))
        .load::<(Uuid, String, String, String, String)>(conn)?;

    let user_ids: Vec<Uuid> = user_rows.iter().map(|(id, _, _, _, _)| *id).collect();

    let roles_rows = if user_ids.is_empty() {
        Vec::new()
    } else {
        ur_dsl::users_roles
            .inner_join(roles_dsl::roles.on(roles_dsl::id.eq(ur_dsl::role_id)))
            .select((ur_dsl::user_id, roles_dsl::name))
            .filter(ur_dsl::user_id.eq_any(&user_ids))
            .load::<(Uuid, String)>(conn)?
    };

    let mut roles_map: HashMap<Uuid, Vec<String>> = HashMap::new();
    for (uid, role_name) in roles_rows {
        roles_map.entry(uid).or_default().push(role_name);
    }

    let users: Vec<UserBasic> = user_rows
        .into_iter()
        .map(|(id, username, email, first_name, last_name)| {
            let roles_for_user = roles_map
                .remove(&id)
                .unwrap_or_else(|| vec!["default".to_string()]);
            UserBasic {
                username,
                email,
                first_name,
                last_name,
                roles: roles_for_user,
            }
        })
        .collect();

    Ok(users)
}

pub fn signin_user(
    conn: &mut PgConnection,
    username_or_email: &str,
    password: &str,
    secret: &str,
) -> Result<(User, String)> {
    use crate::schema::users::dsl::*;
    use anyhow::anyhow;

    let user = users
        .filter(
            username
                .eq(username_or_email)
                .or(email.eq(username_or_email)),
        )
        .first::<User>(conn)
        .map_err(|_| anyhow!("Invalid username/email or password"))?;

    let parsed_hash = argon2::PasswordHash::new(&user.password_hash)
        .map_err(|_| anyhow!("Failed to parse password hash"))?;

    Argon2::default()
        .verify_password(password.as_bytes(), &parsed_hash)
        .map_err(|_| anyhow!("Invalid username/email or password"))?;

    let new_token_version: i32 = rand::thread_rng().gen_range(1..=i32::MAX);
    let updated_user = diesel::update(users.find(user.id))
        .set(token_version.eq(new_token_version))
        .get_result::<User>(conn)?;

    let token = generate_jwt(&updated_user, secret, get_jwt_expire());

    Ok((updated_user, token))
}
