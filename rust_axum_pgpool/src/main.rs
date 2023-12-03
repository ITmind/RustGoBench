use axum::{
    extract::State,
    http::{header::AUTHORIZATION, HeaderMap, StatusCode},
    response::IntoResponse,
    routing::get,
    Json, Router,
};

use deadpool_postgres::{
    Client, Config, ManagerConfig, Pool, PoolConfig, RecyclingMethod, Runtime,
};
use jsonwebtoken::{decode, Algorithm, DecodingKey, Validation};
use serde::{Deserialize, Serialize};
use tokio_postgres::NoTls;

#[derive(Debug, Serialize, Deserialize)]
struct Claims {
    email: String,
    exp: usize,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct User {
    pub email: String,
    pub first: String,
    pub last: String,
    pub city: String,
    pub county: String,
    pub age: i32,
}

type ConnectionPool = Pool;

async fn root(headers: HeaderMap, State(pool): State<ConnectionPool>) -> impl IntoResponse {
    let jwt_secret = "mysuperPUPERsecret100500security";

    let auth_header = headers.get(AUTHORIZATION).expect("no authorization header");
    let mut auth_hdr: &str = auth_header.to_str().unwrap();
    auth_hdr = &auth_hdr.strip_prefix("Bearer ").unwrap();

    let token = decode::<Claims>(
        &auth_hdr,
        &DecodingKey::from_secret(jwt_secret.as_ref()),
        &Validation::new(Algorithm::HS256),
    )
    .ok();

    let client: Client = pool.get().await.unwrap();

    let _email = token.unwrap().claims.email;
    let stmt = client
        .prepare_cached("SELECT * FROM users WHERE email = $1")
        .await
        .unwrap();

    let user = client
        .query_one(&stmt, &[&_email])
        .await
        .map(|row| User {
            email: row.get(0),
            first: row.get(1),
            last: row.get(2),
            city: row.get(3),
            county: row.get(4),
            age: row.get(5),
        })
        .expect("msg");

    (StatusCode::OK, Json(user))
}

#[tokio::main]
async fn main() {
    println!("Starting http server: 127.0.0.1:3000");

    //let database_url = String::from("postgres://postgres:123456@localhost/testbench");

    let mut cfg = Config::new();
    cfg.host = Some("localhost".to_string());
    cfg.dbname = Some("testbench".to_string());
    cfg.user = Some("postgres".to_string());
    cfg.password = Some("123456".to_string());
    cfg.manager = Some(ManagerConfig {
        recycling_method: RecyclingMethod::Fast,
    });
    cfg.pool = Some(PoolConfig::new(12));
    let pool = cfg.create_pool(Some(Runtime::Tokio1), NoTls).unwrap();

    println!("DB connect success");

    let router = Router::new().route("/", get(root)).with_state(pool);

    let listener = tokio::net::TcpListener::bind("0.0.0.0:3000").await.unwrap();
    axum::serve(listener, router).await.unwrap();
}
