#![feature(box_syntax, proc_macro_hygiene, decl_macro)]

#[macro_use] extern crate rocket;
#[macro_use] extern crate rocket_contrib;
#[macro_use] extern crate diesel;
#[macro_use] extern crate serde;
#[macro_use] extern crate failure;
extern crate argon2rs;
extern crate base64;
extern crate multipart;
extern crate chrono;

pub mod util;
pub mod db;
pub mod api;
pub mod model;
pub mod site;

fn main() {
    rocket::ignite()
        .manage(db::Connection::pool())
        // auth module api
        .mount("/api/auth/login", routes![api::authentication::login_http])
        .mount("/api/auth/login", routes![api::authentication::login_json])
        .mount("/api/auth/login", routes![api::authentication::logout])
        .mount("/api/auth/login", routes![api::authentication::query])
        .mount("/api/auth/register", routes![api::authentication::register_http])
        .mount("/api/auth/register", routes![api::authentication::register_json])
        // file module api
        .mount("/api/file", routes![api::app::files::upload_http])
        .mount("/api/file", routes![api::app::files::upload_forbidden])
        .mount("/api/file", routes![api::app::files::delete])
        .mount("/api/file", routes![api::app::files::query])
        .mount("/api/file", routes![api::app::files::query_forbidden])
        // site
        .mount("/", routes![site::files])
        .mount("/", routes![site::index])
        // file lookup mounted here for shorter urls
        .mount("/f", routes![api::app::files::lookup])
        .register(catchers![
            site::json_401,
            site::json_403,
            site::json_404,
            site::json_500
        ])
        .launch();
}
