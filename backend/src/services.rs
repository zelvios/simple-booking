use crate::DbPool;
use actix_web::HttpResponse;
use diesel::pg::PgConnection;
use diesel::r2d2::PooledConnection;

pub fn get_conn(
    pool: &DbPool,
) -> Result<PooledConnection<diesel::r2d2::ConnectionManager<PgConnection>>, HttpResponse> {
    pool.get().map_err(|e| {
        eprintln!("Failed to get DB connection: {}", e);
        HttpResponse::InternalServerError().finish()
    })
}

pub fn get_jwt_secret() -> String {
    std::env::var("JWT_SECRET").expect("JWT_SECRET must be set in the environment")
}
