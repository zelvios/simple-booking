extern crate core;

mod models;
mod schema;
mod services;
mod users;

use crate::users::{
    create_user_endpoint, get_users_endpoint, sign_in_endpoint, update_user_endpoint,
    update_user_password_endpoint, users_verify_token_endpoint,
};
use actix_web::{web, App, HttpServer};
use diesel::pg::PgConnection;
use diesel::r2d2;
use diesel::r2d2::ConnectionManager;
use dotenvy::dotenv;
use std::env;

pub type DbPool = r2d2::Pool<ConnectionManager<PgConnection>>;

#[actix_web::main]
async fn main() -> anyhow::Result<()> {
    dotenv().ok();
    let database_url = env::var("DATABASE_URL")?;

    let manager = ConnectionManager::<PgConnection>::new(&database_url);

    let pool: DbPool = r2d2::Pool::builder()
        .build(manager)
        .expect("Failed to create DB pool.");

    HttpServer::new(move || {
        App::new()
            .app_data(web::Data::new(pool.clone()))
            .service(create_user_endpoint)
            .service(get_users_endpoint)
            .service(sign_in_endpoint)
            .service(users_verify_token_endpoint)
            .service(update_user_endpoint)
            .service(update_user_password_endpoint)
    })
    .bind(("0.0.0.0", 3000))?
    .run()
    .await?;
    Ok(())
}
