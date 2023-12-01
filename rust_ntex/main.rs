//#[cfg(not(target_os = "macos"))]
//#[global_allocator]
//static GLOBAL: snmalloc_rs::SnMalloc = snmalloc_rs::SnMalloc;
// static GLOBAL: mimalloc::MiMalloc = mimalloc::MiMalloc;

use deadpool_postgres::{
    Client, Config, ManagerConfig, Pool, PoolConfig, RecyclingMethod, Runtime,
};
use jsonwebtoken::{decode, Algorithm, DecodingKey, Validation};
use ntex::http::header::AUTHORIZATION;
use ntex::web::{self, HttpRequest};
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

mod errors {
    use deadpool_postgres::PoolError;
    use derive_more::{Display, From};
    use ntex::web::{HttpRequest, HttpResponse, WebResponseError};
    use tokio_postgres::error::Error as PGError;

    #[derive(Display, From, Debug)]
    pub enum MyError {
        NotFound,
        PGError(PGError),
        PoolError(PoolError),
    }
    impl std::error::Error for MyError {}

    impl WebResponseError for MyError {
        fn error_response(&self, _: &HttpRequest) -> HttpResponse {
            match *self {
                MyError::NotFound => HttpResponse::NotFound().finish(),
                MyError::PoolError(ref err) => {
                    HttpResponse::InternalServerError().body(err.to_string())
                }
                _ => HttpResponse::InternalServerError().finish(),
            }
        }
    }
}

async fn root(
    pool: web::types::State<Pool>,
    _req: HttpRequest,
) -> Result<impl web::Responder, errors::MyError> {
    let jwt_secret = "mysuperPUPERsecret100500security";

    let headers: &ntex::http::HeaderMap = _req.headers();
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
            return Err(errors::MyError::NotFound);
        }
    };

    let client: Client = pool.get().await.map_err(errors::MyError::PoolError)?;

    let _email = token.claims.email;
    let stmt = client
        .prepare_cached("SELECT * FROM users WHERE email = $1")
        .await
        .unwrap();

    let row = client.query_one(&stmt, &[&_email]).await.map(|row| User {
        email: row.get(0),
        first: row.get(1),
        last: row.get(2),
        city: row.get(3),
        county: row.get(4),
        age: row.get(5),
    });

    match row {
        Ok(user) => return Ok(web::HttpResponse::Ok().json(&user)),
        Err(_e) => {
            println!("error: {}", _e.to_string());
            return Err(errors::MyError::NotFound);
        }
    };
}

#[ntex::main]
async fn main() -> std::io::Result<()> {
    println!("Starting http server: 127.0.0.1:3000");

    let mut cfg = Config::new();
    cfg.host = Some("localhost".to_string());
    cfg.dbname = Some("testbench".to_string());
    cfg.user = Some("postgres".to_string());
    cfg.password = Some("123456".to_string());
    cfg.manager = Some(ManagerConfig {
        recycling_method: RecyclingMethod::Fast,
    });
    cfg.pool = Some(PoolConfig::new(10));
    let pool = cfg.create_pool(Some(Runtime::Tokio1), NoTls).unwrap();

    {
        let client = pool.get().await.unwrap();
        let stmt = client
            .prepare_cached("SELECT * FROM users WHERE email = $1")
            .await
            .unwrap();
    }

    web::HttpServer::new(move || {
        web::App::new()
            .state(pool.clone())
            .route("/", web::get().to(root))
    })
    .bind(("127.0.0.1", 3000))?
    //.workers(num_cpus::get())
    .run()
    .await
}
