use crate::models::{NewUser, User, UserBasic};
use crate::users::{UpdatePasswordRequest, UpdateUserRequest};
use actix_web::{HttpRequest, HttpResponse};
use anyhow::{anyhow, Result};
use argon2::{password_hash::SaltString, Argon2, PasswordHasher, PasswordVerifier};
use chrono::Utc;
use diesel::prelude::*;
use jsonwebtoken::errors::ErrorKind;
use jsonwebtoken::{decode, encode, DecodingKey, EncodingKey, Header, Validation};
use rand::rngs::OsRng;
use rand::Rng;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::env;
use uuid::Uuid;

#[derive(Serialize, Deserialize)]
struct Claims {
    sub: Uuid,
    username: String,
    token_version: i32,
    exp: i64,
}

pub(crate) fn validate_password(password: &str) -> Result<(), String> {
    if password.len() < 8 {
        return Err("Password must be at least 8 characters long".into());
    }

    if !password.chars().any(|c| c.is_uppercase()) {
        return Err("Password must contain at least one uppercase letter".into());
    }

    if !password.chars().any(|c| c.is_lowercase()) {
        return Err("Password must contain at least one lowercase letter".into());
    }

    if !password.chars().any(|c| c.is_numeric()) {
        return Err("Password must contain at least one number".into());
    }

    let special_chars = "!@#$%^&*()-_=+[]{}|;:'\",.<>?/`~";
    if !password.chars().any(|c| special_chars.contains(c)) {
        return Err("Password must contain at least one special character".into());
    }

    Ok(())
}

pub fn validate_user_fields(
    username: Option<&str>,
    first_name: Option<&str>,
    last_name: Option<&str>,
    email: Option<&str>,
) -> Result<(), String> {
    if username.map(|u| u.len() < 3).unwrap_or(false) {
        return Err("Username must be at least 3 characters long".into());
    }

    if first_name.map(|f| f.len() < 3).unwrap_or(false) {
        return Err("First name must be at least 3 characters long".into());
    }

    if last_name.map(|l| l.len() < 3).unwrap_or(false) {
        return Err("Last name must be at least 3 characters long".into());
    }

    if email.map(|e| e.len() < 3).unwrap_or(false) {
        return Err("Email must be at least 3 characters long".into());
    }

    Ok(())
}

fn generate_new_token_version() -> i32 {
    rand::thread_rng().gen_range(1..=i32::MAX)
}

fn get_jwt_expire() -> i64 {
    env::var("JWT_EXPIRE_SECONDS")
        .ok()
        .and_then(|s| s.parse().ok())
        .unwrap_or(3600) // 1 hour
}

pub fn extract_bearer_token(req: &HttpRequest) -> Result<String, HttpResponse> {
    match req.headers().get("Authorization") {
        Some(hdr_value) => match hdr_value.to_str() {
            Ok(s) if s.starts_with("Bearer ") => Ok(s.trim_start_matches("Bearer ").to_string()),
            Ok(_) => Err(HttpResponse::Unauthorized().body("Invalid Authorization header format")),
            Err(_) => Err(HttpResponse::Unauthorized().body("Invalid header value")),
        },
        None => Err(HttpResponse::Unauthorized().body("Missing Authorization header")),
    }
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

    validate_user_fields(
        Some(&new_user.username),
        Some(&new_user.first_name),
        Some(&new_user.last_name),
        Some(&new_user.email),
    )
    .map_err(|e| anyhow::anyhow!(e))?;

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

    let new_token_version: i32 = generate_new_token_version();
    let updated_user = diesel::update(users.find(user.id))
        .set(token_version.eq(new_token_version))
        .get_result::<User>(conn)?;

    let token = generate_jwt(&updated_user, secret, get_jwt_expire());

    Ok((updated_user, token))
}

pub fn verify_token(conn: &mut PgConnection, token: &str, secret: &str) -> Result<(bool, String)> {
    use crate::schema::users::dsl::*;

    match decode::<Claims>(
        token,
        &DecodingKey::from_secret(secret.as_ref()),
        &Validation::default(),
    ) {
        Ok(token_data) => {
            let claims = token_data.claims;
            match users.find(claims.sub).first::<User>(conn).optional()? {
                Some(user) => {
                    if user.token_version == claims.token_version {
                        Ok((true, "ok".to_string()))
                    } else {
                        Ok((false, "token_version_mismatch".to_string()))
                    }
                }
                None => Ok((false, "user_not_found".to_string())),
            }
        }
        Err(err) => match *err.kind() {
            ErrorKind::ExpiredSignature => Ok((false, "expired".to_string())),
            _ => Ok((false, "invalid".to_string())),
        },
    }
}

pub fn update_user(
    conn: &mut PgConnection,
    token: &str,
    secret: &str,
    data: UpdateUserRequest,
) -> Result<User> {
    use crate::schema::users::dsl::*;

    let claims = decode::<Claims>(
        token,
        &DecodingKey::from_secret(secret.as_ref()),
        &Validation::default(),
    )?
    .claims;

    let current_token_version: i32 = users
        .filter(id.eq(claims.sub))
        .select(token_version)
        .first(conn)?;

    if current_token_version != claims.token_version {
        return Err(anyhow::anyhow!("Invalid or expired token"));
    }

    if let Some(ref new_username) = data.username {
        if username_exists(conn, new_username)? {
            return Err(anyhow::anyhow!("Username already taken"));
        }
    }

    if let Some(ref new_email) = data.email {
        if email_exists(conn, new_email)? {
            return Err(anyhow::anyhow!("Email already in use"));
        }
    }

    validate_user_fields(
        data.username.as_deref(),
        data.first_name.as_deref(),
        data.last_name.as_deref(),
        data.email.as_deref(),
    )
    .map_err(|e| anyhow::anyhow!(e))?;

    let changes = crate::models::UpdateUserChangeset {
        username: data.username,
        email: data.email,
        first_name: data.first_name,
        last_name: data.last_name,
    };

    let updated_user = diesel::update(users.find(claims.sub))
        .set(&changes)
        .get_result::<User>(conn)?;

    Ok(updated_user)
}

pub fn update_password(
    conn: &mut PgConnection,
    token: &str,
    secret: &str,
    data: UpdatePasswordRequest,
) -> Result<()> {
    use crate::schema::users::dsl::*;

    let claims = decode::<Claims>(
        token,
        &DecodingKey::from_secret(secret.as_ref()),
        &Validation::default(),
    )?
    .claims;

    let (current_token_version, current_password_hash): (i32, String) = users
        .filter(id.eq(claims.sub))
        .select((token_version, password_hash))
        .first(conn)?;

    if current_token_version != claims.token_version {
        return Err(anyhow::anyhow!("Invalid or expired token"));
    }

    let parsed_hash = argon2::PasswordHash::new(&current_password_hash)
        .map_err(|e| anyhow::anyhow!("Failed to parse password hash: {}", e))?;
    Argon2::default()
        .verify_password(data.old_password.as_bytes(), &parsed_hash)
        .map_err(|_| anyhow::anyhow!("Invalid old password"))?;

    if Argon2::default()
        .verify_password(data.new_password.as_bytes(), &parsed_hash)
        .is_ok()
    {
        return Err(anyhow::anyhow!(
            "New password cannot be the same as the old password"
        ));
    }

    validate_password(&data.new_password)
        .map_err(|e| anyhow::anyhow!("New password validation failed: {}", e))?;

    let new_hash = hash_password(&data.new_password)
        .map_err(|_| anyhow::anyhow!("Failed to hash new password"))?;

    let new_version: i32 = generate_new_token_version();

    let changes = crate::models::UpdatePasswordChangeset {
        password_hash: new_hash,
        token_version: new_version,
    };

    diesel::update(users.find(claims.sub))
        .set(&changes)
        .execute(conn)?;

    Ok(())
}
