use crate::models::NewUser;
use crate::{services, DbPool};
use actix_web::{post, web, HttpResponse};
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

#[post("/users")]
pub async fn create_user_endpoint(
    pool: web::Data<DbPool>,
    body: web::Json<CreateUserRequest>,
) -> HttpResponse {
    let mut conn = match services::get_conn(&pool) {
        Ok(c) => c,
        Err(resp) => return resp,
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
                "id": user.id,
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
