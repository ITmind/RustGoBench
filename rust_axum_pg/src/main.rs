mod db;
mod server;

use axum::{
    http::{header::AUTHORIZATION, HeaderMap, StatusCode},
    response::IntoResponse,
    routing::get,
    Json, Router,
};

use jsonwebtoken::{decode, Algorithm, DecodingKey, Validation};
use serde::{Deserialize, Serialize};

use crate::db::{DatabaseConnection, PgConnection, User};

#[derive(Debug, Serialize, Deserialize)]
struct Claims {
    email: String,
    exp: usize,
}

async fn root(
    headers: HeaderMap,
    DatabaseConnection(conn): DatabaseConnection,
) -> impl IntoResponse {
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

    let _email = token.unwrap().claims.email;
    let user = conn.get_user(_email).await.expect("error loading world");
    (StatusCode::OK, Json(user))
}

//#[tokio::main]
fn main() {
    let rt = tokio::runtime::Builder::new_current_thread()
        .enable_all()
        .build()
        .unwrap();

    for _ in 1..num_cpus::get() {
        std::thread::spawn(move || {
            let rt = tokio::runtime::Builder::new_current_thread()
                .enable_all()
                .build()
                .unwrap();
            rt.block_on(serve());
        });
    }
    rt.block_on(serve());
}

async fn serve() {
    let database_url = String::from("postgres://postgres:123456@localhost/testbench");
    let pg_connection = PgConnection::connect(database_url).await;

    println!("DB connect success");

    let router = Router::new()
        .route("/", get(root))
        .with_state(pg_connection);

    server::builder()
        .serve(router.into_make_service())
        .await
        .unwrap();
}
