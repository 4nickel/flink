use std::ops::Deref;

use diesel::prelude::{RunQueryDsl, SqliteConnection};
use diesel::r2d2::{ConnectionManager, CustomizeConnection, Pool, PooledConnection};

use rocket::http::Status;
use rocket::request::{self, FromRequest};
use rocket::{Outcome, Request, State};

pub const DATABASE_URL: &'static str = env!("DATABASE_URL");

pub type SqlitePool = Pool<ConnectionManager<SqliteConnection>>;
pub struct Connection(pub PooledConnection<ConnectionManager<SqliteConnection>>);

#[derive(Debug)]
struct SqliteConnectionCustomizer();
impl<C: diesel::Connection, E> CustomizeConnection<C, E> for SqliteConnectionCustomizer {
    fn on_acquire(&self, connection: &mut C) -> Result<(), E> {
        diesel::dsl::sql_query(format!("PRAGMA foreign_keys = ON"))
            .execute(connection)
            .expect("Pragma Error");
        Ok(())
    }
}

impl Connection {
    pub fn pool() -> SqlitePool {
        let manager = ConnectionManager::<SqliteConnection>::new(DATABASE_URL);
        Pool::builder()
            .max_size(8)
            .connection_customizer(box SqliteConnectionCustomizer {})
            .build(manager)
            .expect("[database] error building connection pool")
    }
}

impl Deref for Connection {
    type Target = SqliteConnection;

    #[inline(always)]
    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl<'a, 'r> FromRequest<'a, 'r> for Connection {
    type Error = ();

    fn from_request(request: &'a Request<'r>) -> request::Outcome<Connection, ()> {
        let pool = request.guard::<State<SqlitePool>>()?;
        match pool.get() {
            Ok(c) => {
                println!("[database] acquired connection");
                Outcome::Success(Connection(c))
            }
            Err(_) => {
                println!("[database] failed to connect");
                Outcome::Failure((Status::ServiceUnavailable, ()))
            }
        }
    }
}
