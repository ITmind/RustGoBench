//#[cfg(not(target_os = "macos"))]
//#[global_allocator]
//static GLOBAL: snmalloc_rs::SnMalloc = snmalloc_rs::SnMalloc;
// static GLOBAL: mimalloc::MiMalloc = mimalloc::MiMalloc;

use jsonwebtoken::{decode, Algorithm, DecodingKey, Validation};
use ntex::http::header::{HeaderValue, AUTHORIZATION};
use ntex::http::header::{CONTENT_TYPE, SERVER};
use ntex::http::{HttpService, KeepAlive, Request, Response, StatusCode};
use ntex::service::{Service, ServiceCtx, ServiceFactory};
use ntex::web::{self, HttpRequest};
use ntex::web::{Error, HttpResponse};
use ntex::{time::Seconds, util::BoxFuture, util::PoolId};
use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
struct Claims {
    email: String,
    exp: usize,
}

mod db;

const JWT_SECRET: &'static str = "mysuperPUPERsecret100500security";

struct App(db::PgConnection);

impl Service<Request> for App {
    type Response = Response;
    type Error = Error;
    type Future<'f> = BoxFuture<'f, Result<Response, Error>> where Self: 'f;

    fn call<'a>(&'a self, req: Request, _: ServiceCtx<'a, Self>) -> Self::Future<'a> {
        Box::pin(async move {
            match req.path() {
                "/" => {
                    let headers: &ntex::http::HeaderMap = req.headers();
                    let auth_header = headers.get(AUTHORIZATION).expect("no authorization header");
                    let mut auth_hdr: &str = auth_header.to_str().unwrap();
                    auth_hdr = &auth_hdr.strip_prefix("Bearer ").unwrap();

                    let token = decode::<Claims>(
                        &auth_hdr,
                        &DecodingKey::from_secret(JWT_SECRET.as_ref()),
                        &Validation::new(Algorithm::HS256),
                    )
                    .ok();

                    let _email = token.unwrap().claims.email;
                    let body = self.0.get_user(_email).await;
                    let mut res = HttpResponse::Ok().json(&body);
                    res.headers_mut()
                        .insert(SERVER, HeaderValue::from_static("N"));
                    res.headers_mut()
                        .insert(CONTENT_TYPE, HeaderValue::from_static("application/json"));
                    Ok(res)
                }
                _ => Ok(Response::new(StatusCode::NOT_FOUND)),
            }
        })
    }
}

struct AppFactory;

impl ServiceFactory<Request> for AppFactory {
    type Response = Response;
    type Error = Error;
    type Service = App;
    type InitError = ();
    type Future<'f> = BoxFuture<'f, Result<Self::Service, Self::InitError>>;

    fn create(&self, _: ()) -> Self::Future<'_> {
        const DB_URL: &str = "postgres://postgres:123456@localhost/testbench";

        Box::pin(async move { Ok(App(db::PgConnection::connect(DB_URL).await)) })
    }
}

#[ntex::main]
async fn main() -> std::io::Result<()> {
    println!(
        "Starting http server: 127.0.0.1:3000 cpu:{}",
        num_cpus::get()
    );

    ntex::server::build()
        .backlog(1024)
        .bind("techempower", "0.0.0.0:3000", |cfg| {
            cfg.memory_pool(PoolId::P1);
            PoolId::P1.set_read_params(65535, 2048);
            PoolId::P1.set_write_params(65535, 2048);

            HttpService::build()
                .keep_alive(KeepAlive::Os)
                .client_timeout(Seconds(0))
                .h1(AppFactory)
        })?
        .workers(num_cpus::get())
        .run()
        .await
}
