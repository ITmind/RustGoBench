use std::{convert::Infallible, io, sync::Arc};

use axum::{async_trait, extract::FromRequestParts, http::request::Parts};
use futures::StreamExt;
use serde::{Deserialize, Serialize};
use tokio::pin;
use tokio_postgres::{connect, Client, NoTls, Statement};

#[derive(Debug)]
pub enum PgError {
    Io(io::Error),
    Pg(tokio_postgres::Error),
}

impl From<io::Error> for PgError {
    fn from(err: io::Error) -> Self {
        PgError::Io(err)
    }
}

impl From<tokio_postgres::Error> for PgError {
    fn from(err: tokio_postgres::Error) -> Self {
        PgError::Pg(err)
    }
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

/// Postgres interface
pub struct PgConnection {
    client: Client,
    users: Statement,
}

impl PgConnection {
    pub async fn connect(db_url: String) -> Arc<PgConnection> {
        let (cl, conn) = connect(&db_url, NoTls)
            .await
            .expect("can not connect to postgresql");

        // Spawn connection
        tokio::spawn(async move {
            if let Err(error) = conn.await {
                eprintln!("Connection error: {error}");
            }
        });

        let users = cl
            .prepare("SELECT * FROM users WHERE email = $1")
            .await
            .unwrap();

        Arc::new(PgConnection { client: cl, users })
    }
}

impl PgConnection {
    pub async fn get_user(&self, email: String) -> Result<User, PgError> {
        let stream = self.client.query_raw(&self.users, &[&email]).await?;
        pin!(stream);
        let row = stream.next().await.unwrap()?;
        Ok(User {
            email: row.get(0),
            first: row.get(1),
            last: row.get(2),
            city: row.get(3),
            county: row.get(4),
            age: row.get(5),
        })
    }
}

pub struct DatabaseConnection(pub Arc<PgConnection>);

#[async_trait]
impl FromRequestParts<Arc<PgConnection>> for DatabaseConnection {
    type Rejection = Infallible;

    async fn from_request_parts(
        _parts: &mut Parts,
        pg_connection: &Arc<PgConnection>,
    ) -> Result<Self, Self::Rejection> {
        Ok(Self(pg_connection.clone()))
    }
}
