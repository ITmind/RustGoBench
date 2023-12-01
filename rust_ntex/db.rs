use std::{cell::RefCell, fmt::Write as FmtWrite};

use ntex::util::BytesMut;
use serde::{Deserialize, Serialize};
use tokio_postgres::{connect, Client, NoTls, Statement};

#[derive(Debug, Deserialize, Serialize)]
pub struct User {
    pub email: String,
    pub first: String,
    pub last: String,
    pub city: String,
    pub county: String,
    pub age: i32,
}

/// Postgres interface
pub struct PgConnection {
    cl: Client,
    user: Statement,
    buf: RefCell<BytesMut>,
}

impl PgConnection {
    pub async fn connect(db_url: &str) -> PgConnection {
        let (cl, conn) = connect(db_url, NoTls)
            .await
            .expect("can not connect to postgresql");
        ntex::rt::spawn(async move {
            let _ = conn.await;
        });

        let user = cl
            .prepare("SELECT * FROM users WHERE email = $1")
            .await
            .unwrap();

        PgConnection {
            cl,
            user,
            buf: RefCell::new(BytesMut::with_capacity(65535)),
        }
    }
}

impl PgConnection {
    pub async fn get_user(&self, email: String) -> User {
        let row = self.cl.query_one(&self.user, &[&email]).await.unwrap();

        //let mut body = self.buf.borrow_mut();
        User {
            email: row.get(0),
            first: row.get(1),
            last: row.get(2),
            city: row.get(3),
            county: row.get(4),
            age: row.get(5),
        }
    }
}
