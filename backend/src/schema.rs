// @generated automatically by Diesel CLI.

pub mod sql_types {
    #[derive(diesel::query_builder::QueryId, Clone, diesel::sql_types::SqlType)]
    #[diesel(postgres_type(name = "booking_status"))]
    pub struct BookingStatus;
}

diesel::table! {
    use diesel::sql_types::*;
    use super::sql_types::BookingStatus;

    bookings (id) {
        id -> Uuid,
        user_id -> Uuid,
        #[max_length = 255]
        title -> Varchar,
        description -> Nullable<Text>,
        booking_date -> Timestamptz,
        status -> BookingStatus,
        created_at -> Timestamptz,
        updated_at -> Timestamptz,
        deleted_at -> Nullable<Timestamptz>,
    }
}

diesel::table! {
    permissions (id) {
        id -> Int4,
        #[max_length = 50]
        name -> Varchar,
    }
}

diesel::table! {
    roles (id) {
        id -> Int4,
        #[max_length = 50]
        name -> Varchar,
    }
}

diesel::table! {
    roles_permissions (role_id, permission_id) {
        role_id -> Int4,
        permission_id -> Int4,
    }
}

diesel::table! {
    users (id) {
        id -> Uuid,
        #[max_length = 40]
        username -> Varchar,
        #[max_length = 100]
        first_name -> Varchar,
        #[max_length = 100]
        last_name -> Varchar,
        #[max_length = 255]
        email -> Varchar,
        password_hash -> Text,
        token_version -> Int4,
        locked_until -> Nullable<Timestamptz>,
        last_login_at -> Nullable<Timestamptz>,
        deleted_at -> Nullable<Timestamptz>,
        created_at -> Timestamptz,
        updated_at -> Timestamptz,
    }
}

diesel::table! {
    users_roles (user_id, role_id) {
        user_id -> Uuid,
        role_id -> Int4,
    }
}

diesel::joinable!(bookings -> users (user_id));
diesel::joinable!(roles_permissions -> permissions (permission_id));
diesel::joinable!(roles_permissions -> roles (role_id));
diesel::joinable!(users_roles -> roles (role_id));
diesel::joinable!(users_roles -> users (user_id));

diesel::allow_tables_to_appear_in_same_query!(
    bookings,
    permissions,
    roles,
    roles_permissions,
    users,
    users_roles,
);
