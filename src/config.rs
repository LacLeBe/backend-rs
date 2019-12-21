use diesel::mysql::MysqlConnection;
use diesel::r2d2::{ConnectionManager, Pool, PooledConnection};
use rocket_cors;

use rocket::http::Status;
use rocket::request::{self, FromRequest};
use rocket::{Outcome, Request, State};
use rocket_cors::Cors;
use std::ops::Deref;

use super::routes;
use dotenv::dotenv;
use std::env;

use crate::models;

// type of connection
type MysqlPool = Pool<ConnectionManager<MysqlConnection>>;

// -> Init database pool/conn
fn init_pool(db_url: String) -> MysqlPool {
    let connect = ConnectionManager::<MysqlConnection>::new(db_url);
    Pool::new(connect).expect("Failed to create pool!")
}

// -> Handle CORS
fn enable_cors() -> Cors {
    Cors::from_options(&Default::default()).expect("Cors can't be created")
}

// -> Launch App
pub fn connect_db() -> rocket::Rocket {
    dotenv().ok();
    let database_url = env::var("DATABASE_URL").expect("DATABASE_URL must be set!");
    let pool = init_pool(database_url);

    rocket::ignite()
        .manage(pool)
        .mount(
            "/",
            routes![
                routes::user_route::index,
                routes::user_route::view_users,
                routes::user_route::create_user
            ],
        )
        .attach(enable_cors())
        .register(catchers![routes::not_found])
}

// -> Request Handler
pub struct Connection(pub PooledConnection<ConnectionManager<MysqlConnection>>);

impl<'a, 'r> FromRequest<'a, 'r> for Connection {
    type Error = ();

    fn from_request(request: &'a Request<'r>) -> request::Outcome<Self, Self::Error> {
        let pool = request.guard::<State<MysqlPool>>()?;
        match pool.get() {
            Ok(conn) => Outcome::Success(Connection(conn)),
            Err(_) => Outcome::Failure((Status::ServiceUnavailable, ())),
        }
    }
}

// For the convenience of using an &Connection as an &MysqlConnection.
impl Deref for Connection {
    type Target = MysqlConnection;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}
