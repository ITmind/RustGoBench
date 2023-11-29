use axum::{
    extract::State,
    http::{header::AUTHORIZATION, HeaderMap, StatusCode},
    response::IntoResponse,
    routing::get,
    Json, Router,
};
use jsonwebtoken::{decode, Algorithm, DecodingKey, Validation};
use serde::{Deserialize, Serialize};
use sqlx::{postgres::PgPoolOptions, Pool, Postgres};

#[derive(Debug, Serialize, Deserialize)]
struct Claims {
    email: String,
    exp: usize,
}

#[derive(Debug, Deserialize, Serialize, sqlx::FromRow)]
pub struct User {
    pub email: String,
    pub first: Option<String>,
    pub last: Option<String>,
    pub city: Option<String>,
    pub county: Option<String>,
    pub age: Option<i32>,
}

type ConnectionPool = Pool<Postgres>;

async fn root(headers: HeaderMap, State(pool): State<ConnectionPool>) -> impl IntoResponse {
    let jwt_secret = "mysuperPUPERsecret100500security";
    let validation = Validation::new(Algorithm::HS256);

    let auth_header = headers.get(AUTHORIZATION).expect("no authorization header");
    let mut auth_hdr: &str = auth_header.to_str().unwrap();
    auth_hdr = &auth_hdr.strip_prefix("Bearer ").unwrap();

    let token = match decode::<Claims>(
        &auth_hdr,
        &DecodingKey::from_secret(jwt_secret.as_ref()),
        &validation,
    ) {
        Ok(c) => c,
        Err(e) => {
            eprintln!("Application error: {e}");
            return (StatusCode::INTERNAL_SERVER_ERROR, "invalid token").into_response();
        }
    };

    let email = token.claims.email;
    // let email = String::from("madeline_dolores@hotmail.com");
    let query_result: Result<User, sqlx::Error> =
        sqlx::query_as(r#"SELECT *  FROM USERS WHERE email=$1"#)
            .bind(email)
            .fetch_one(&pool)
            .await;

    match query_result {
        Ok(user) => {
            return (StatusCode::ACCEPTED, Json(user)).into_response();
        }
        Err(sqlx::Error::RowNotFound) => {
            return (StatusCode::NOT_FOUND, "user not found").into_response();
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
    let pool = PgPoolOptions::new()
        .max_connections(10)
        .connect(&database_url)
        .await
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
