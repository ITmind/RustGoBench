mod schema;

use axum::{
    extract::State,
    http::{header::AUTHORIZATION, HeaderMap, StatusCode},
    response::IntoResponse,
    routing::get,
    Json, Router,
};
use diesel::{
    prelude::*,
    r2d2::{ConnectionManager, Pool},
};
use jsonwebtoken::{decode, Algorithm, DecodingKey, Validation};
use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
struct Claims {
    email: String,
    exp: usize,
}

#[derive(Debug, Deserialize, Serialize, Queryable, Selectable)]
#[diesel(table_name = crate::schema::users)]
pub struct User {
    pub email: String,
    pub first: String,
    pub last: String,
    pub city: String,
    pub county: String,
    pub age: i32,
}

type ConnectionPool = Pool<ConnectionManager<PgConnection>>;

async fn root(headers: HeaderMap, State(pool): State<ConnectionPool>) -> impl IntoResponse {
    let jwt_secret = "mysuperPUPERsecret100500security";

    let auth_header = headers.get(AUTHORIZATION).expect("no authorization header");
    let mut auth_hdr: &str = auth_header.to_str().unwrap();
    auth_hdr = &auth_hdr.strip_prefix("Bearer ").unwrap();

    let token = match decode::<Claims>(
        &auth_hdr,
        &DecodingKey::from_secret(jwt_secret.as_ref()),
        &Validation::new(Algorithm::HS256),
    ) {
        Ok(c) => c,
        Err(e) => {
            println!("Application error: {e}");
            return (StatusCode::INTERNAL_SERVER_ERROR, "invalid token").into_response();
        }
    };

    let conn = &mut pool.get().unwrap();

    use crate::schema::users::dsl::*;
    let _email = token.claims.email;
    let query_result = users
        .filter(email.eq(_email))
        .limit(1)
        .select(User::as_select())
        .first(conn);

    match query_result {
        Ok(user) => {
            return (StatusCode::ACCEPTED, Json(user)).into_response();
        }
        Err(_e) => {
            println!("error: {}", _e.to_string());
            return (StatusCode::INTERNAL_SERVER_ERROR, "error").into_response();
        }
    };
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let database_url = String::from("postgres://postgres:123456@localhost/testbench");
    let manager = ConnectionManager::<PgConnection>::new(database_url);
    let pool = Pool::builder()
        .max_size(10)
        .min_idle(Some(10))
        .test_on_check_out(false)
        .build(manager)
        .expect("can't connect to database");

    println!("DB connect success");

    let app = Router::new().route("/", get(root)).with_state(pool);

    let listener = tokio::net::TcpListener::bind("127.0.0.1:3000")
        .await
        .unwrap();
    axum::serve(listener, app).await?;
    Ok(())
}

// NOTE ----
// Rust code has been built in release mode for all performance tests
