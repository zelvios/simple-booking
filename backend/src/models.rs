use chrono::{DateTime, Utc};
use diesel::backend::RawValue;
use diesel::deserialize::FromSql;
use diesel::pg::Pg;
use diesel::prelude::*;
use diesel::serialize::{IsNull, Output, ToSql};
use diesel::{
    deserialize, serialize, AsExpression, Associations, FromSqlRow, Identifiable, Insertable,
    Queryable,
};
use serde::{Deserialize, Serialize};
use std::io::Write;
use uuid::Uuid;

use crate::schema::sql_types::BookingStatus as BookingStatusSql;
use crate::schema::{bookings, users};

#[derive(Debug, Clone, Copy, Serialize, Deserialize, FromSqlRow, AsExpression)]
#[diesel(sql_type = BookingStatusSql)]
pub enum BookingStatus {
    Pending,
    Confirmed,
    Cancelled,
    Completed,
    NoShow,
    Delayed,
}

impl FromSql<BookingStatusSql, Pg> for BookingStatus {
    fn from_sql(value: RawValue<'_, Pg>) -> deserialize::Result<Self> {
        match value.as_bytes() {
            b"pending" => Ok(BookingStatus::Pending),
            b"confirmed" => Ok(BookingStatus::Confirmed),
            b"cancelled" => Ok(BookingStatus::Cancelled),
            b"completed" => Ok(BookingStatus::Completed),
            b"no_show" => Ok(BookingStatus::NoShow),
            b"delayed" => Ok(BookingStatus::Delayed),
            other => Err(format!("Unrecognized booking status: {:?}", other).into()),
        }
    }
}

impl ToSql<BookingStatusSql, Pg> for BookingStatus {
    fn to_sql<'b>(&'b self, out: &mut Output<'b, '_, Pg>) -> serialize::Result {
        let s = match self {
            BookingStatus::Pending => "pending",
            BookingStatus::Confirmed => "confirmed",
            BookingStatus::Cancelled => "cancelled",
            BookingStatus::Completed => "completed",
            BookingStatus::NoShow => "no_show",
            BookingStatus::Delayed => "delayed",
        };
        out.write_all(s.as_bytes())?;
        Ok(IsNull::No)
    }
}

#[derive(Debug, Queryable, Identifiable, Associations, Serialize, Deserialize)]
#[diesel(belongs_to(User))]
#[diesel(table_name = bookings)]
pub struct Booking {
    pub id: Uuid,
    pub user_id: Uuid,
    pub title: String,
    pub description: Option<String>,
    pub booking_date: DateTime<Utc>,
    pub status: BookingStatus,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    pub deleted_at: Option<DateTime<Utc>>,
}

#[derive(Debug, Queryable, Identifiable, Serialize, Deserialize)]
#[diesel(table_name = users)]
pub struct User {
    pub id: Uuid,
    pub username: String,
    pub first_name: String,
    pub last_name: String,
    pub email: String,
    pub password_hash: String,
    pub token_version: i32,
    pub locked_until: Option<DateTime<Utc>>,
    pub last_login_at: Option<DateTime<Utc>>,
    pub deleted_at: Option<DateTime<Utc>>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Insertable)]
#[diesel(table_name = users)]
pub struct NewUser {
    pub first_name: String,
    pub last_name: String,
    pub username: String,
    pub email: String,
    pub password_hash: String,
    pub token_version: i32,
}
