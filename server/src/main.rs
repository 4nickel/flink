#![feature(box_syntax, proc_macro_hygiene, decl_macro)]

#[macro_use]
extern crate rocket;
#[macro_use]
extern crate rocket_contrib;
#[macro_use]
extern crate diesel;
#[macro_use]
extern crate serde;
#[macro_use]
extern crate failure;
extern crate argon2rs;
extern crate base64;
extern crate chrono;
extern crate clap;
extern crate multipart;

pub mod api;
pub mod db;
pub mod model;
pub mod site;
pub mod util;

#[derive(Debug)]
enum UserCommand<'a> {
    Add(&'a str, &'a str),
    Del(&'a str),
}

#[derive(Debug)]
enum Command<'a> {
    User(UserCommand<'a>),
    Run,
}

fn launch_rocket() {
    rocket::ignite()
        .manage(db::Connection::pool())
        // auth module api
        .mount("/api/auth/login", routes![api::authentication::login_http])
        .mount("/api/auth/login", routes![api::authentication::login_json])
        .mount("/api/auth/login", routes![api::authentication::logout])
        .mount("/api/auth/login", routes![api::authentication::query])
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

fn add_user(name: &str, password: &str) {
    println!("add user: {}", name);
    use model::{User, UserInsert};
    let pool = db::Connection::pool();
    let connection = db::Connection(pool.get().unwrap());
    User::create(&UserInsert { name: name.into() }, password, &connection).unwrap();
    println!("success");
}

fn del_user(name: &str) {
    println!("del user: {}", name);
    use model::User;
    let pool = db::Connection::pool();
    let connection = db::Connection(pool.get().unwrap());
    if let Ok(user) = User::by_name(name, &connection) {
        User::delete(user.id, &connection).unwrap();
        println!("success");
    } else {
        println!("no such user: {}", name);
    }
}

fn main() {
    use clap::{App, Arg, SubCommand};
    use util::arg::Opt;

    let args = App::new("flink")
        .version("0.1")
        .about("Self-Hosted File-Uploader")
        .author("Felix V.")
        .subcommand(
            SubCommand::with_name("user")
                .about("User subcommand")
                .subcommand(
                    SubCommand::with_name("add")
                        .about("Add a user")
                        .arg(
                            Arg::with_name("NAME")
                                .help("The users name")
                                .required(true)
                                .takes_value(true),
                        )
                        .arg(
                            Arg::with_name("PASS")
                                .help("The users password")
                                .required(true)
                                .takes_value(true),
                        ),
                )
                .subcommand(
                    SubCommand::with_name("del").about("Delete a user").arg(
                        Arg::with_name("NAME")
                            .help("The users name")
                            .required(true)
                            .takes_value(true),
                    ),
                ),
        )
        .subcommand(SubCommand::with_name("run").about("Run the service"))
        .get_matches();

    let oo;
    let command = {
        if let Some(options) = args.subcommand_matches("user") {
            if let Some(options) = options.subcommand_matches("add") {
                oo = Opt::new(options);
                Command::User(UserCommand::Add(oo.get("NAME"), oo.get("PASS")))
            } else if let Some(options) = options.subcommand_matches("del") {
                oo = Opt::new(options);
                Command::User(UserCommand::Del(oo.get("NAME")))
            } else {
                panic!()
            }
        } else if let Some(_options) = args.subcommand_matches("run") {
            Command::Run
        } else {
            panic!()
        }
    };

    match command {
        Command::Run => {
            launch_rocket();
        }
        Command::User(subcommand) => match subcommand {
            UserCommand::Add(name, password) => {
                add_user(name, password);
            }
            UserCommand::Del(name) => {
                del_user(name);
            }
        },
    }
}
