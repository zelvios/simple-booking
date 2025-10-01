use crate::models::NewUser;
use crate::{services, DbPool};
use actix_web::{get, post, web, HttpResponse};
use serde::Deserialize;

pub mod service;

#[derive(Deserialize)]
pub struct CreateUserRequest {
    pub first_name: String,
    pub last_name: String,
    pub username: String,
    pub email: String,
    pub password: String,
}

#[derive(Deserialize)]
pub struct SignInRequest {
    pub username_or_email: String,
    pub password: String,
}

#[derive(Deserialize)]
pub struct VerifyTok {
    pub token: Option<String>,
}

#[post("/users")]
pub async fn create_user_endpoint(
    pool: web::Data<DbPool>,
    body: web::Json<CreateUserRequest>,
) -> HttpResponse {
    let mut conn = match services::get_conn(&pool) {
        Ok(c) => c,
        Err(err) => return err,
    };
    let secret = services::get_jwt_secret();

    let password_hash = match service::hash_password(&body.password) {
        Ok(hash) => hash,
        Err(e) => {
            eprintln!("Password hashing failed: {}", e);
            return HttpResponse::InternalServerError().finish();
        }
    };

    let new_user = NewUser {
        first_name: body.first_name.clone(),
        last_name: body.last_name.clone(),
        username: body.username.clone(),
        email: body.email.clone(),
        password_hash,
        token_version: 0,
    };

    match web::block(move || service::create_user(&mut conn, new_user, &secret)).await {
        Ok(Ok((user, token))) => HttpResponse::Ok().json(serde_json::json!({
            "user": {
                "username": user.username,
                "email": user.email,
            },
            "token": token
        })),
        Ok(Err(e)) => {
            eprintln!("Create user error: {}", e);
            HttpResponse::InternalServerError().body(format!("Error creating user: {}", e))
        }
        Err(e) => {
            eprintln!("Blocking error: {}", e);
            HttpResponse::InternalServerError().body("Error creating user")
        }
    }
}

#[get("/users")]
pub async fn get_users_endpoint(pool: web::Data<DbPool>) -> HttpResponse {
    let mut conn = match services::get_conn(&pool) {
        Ok(c) => c,
        Err(err) => return err,
    };

    match web::block(move || service::get_users(&mut conn)).await {
        Ok(Ok(users)) => HttpResponse::Ok().json(
            users
                .into_iter()
                .map(|user| {
                    serde_json::json!({
                        "username": user.username,
                        "email": user.email,
                        "first_name": user.first_name,
                        "last_name": user.last_name,
                        "roles": user.roles,
                    })
                })
                .collect::<Vec<_>>(),
        ),
        Ok(Err(e)) => {
            eprintln!("DB query error: {}", e);
            HttpResponse::InternalServerError().body("Error fetching users")
        }
        Err(e) => {
            eprintln!("Blocking error: {}", e);
            HttpResponse::InternalServerError().body("Blocking error")
        }
    }
}

#[post("/sign-in")]
pub async fn sign_in_endpoint(
    pool: web::Data<DbPool>,
    body: web::Json<SignInRequest>,
) -> HttpResponse {
    let mut conn = match services::get_conn(&pool) {
        Ok(c) => c,
        Err(err) => return err,
    };
    let secret = services::get_jwt_secret();

    match web::block(move || {
        service::signin_user(&mut conn, &body.username_or_email, &body.password, &secret)
    })
    .await
    {
        Ok(Ok((user, token))) => HttpResponse::Ok().json(serde_json::json!({
            "user": {
                "username": user.username,
                "email": user.email,
                "first_name": user.first_name,
                "last_name": user.last_name,
            },
            "token": token
        })),
        Ok(Err(e)) => {
            eprintln!("Sign-in error: {}", e);
            HttpResponse::Unauthorized().body("Invalid username/email or password {}")
        }
        Err(e) => {
            eprintln!("Blocking error: {}", e);
            HttpResponse::InternalServerError().body("Error signing in")
        }
    }
}

#[post("/users/verify/token")]
pub async fn users_verify_token_endpoint(
    pool: web::Data<DbPool>,
    body: web::Json<VerifyTok>,
) -> HttpResponse {
    let token = match &body.token {
        Some(t) => t.clone(),
        None => {
            return HttpResponse::BadRequest()
                .json(serde_json::json!({"valid": false, "reason": "no_token"}));
        }
    };

    let mut conn = match services::get_conn(&pool) {
        Ok(c) => c,
        Err(resp) => return resp,
    };
    let secret = services::get_jwt_secret();

    match web::block(move || service::verify_token(&mut conn, &token, &secret)).await {
        Ok(Ok((true, _))) => HttpResponse::Ok().json(serde_json::json!({"valid": true})),
        Ok(Ok((false, reason))) => HttpResponse::Ok().json(serde_json::json!({
            "valid": false,
            "reason": reason
        })),
        Ok(Err(e)) => {
            eprintln!("verify_token error: {}", e);
            HttpResponse::InternalServerError().body("Error verifying token")
        }
        Err(e) => {
            eprintln!("blocking error: {}", e);
            HttpResponse::InternalServerError().body("Error verifying token")
        }
    }
}
