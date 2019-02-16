extern crate actix_web;
extern crate clap;
extern crate failure;
extern crate futures;
extern crate rand;
extern crate redis;
extern crate serde;
extern crate serde_derive;
extern crate serde_json;

use std::env::var;
use std::sync::Arc;
use std::sync::Mutex;

use actix_web::http::header::{HeaderName, HeaderValue};
use actix_web::http::Method;
use actix_web::middleware;
use actix_web::{server, App};

use redis::{Client, ConnectionAddr, ConnectionInfo};

mod error;
mod helpers;
mod routes;
mod state;
mod token;

use error::MainError;
use routes::{delete_token, get_token, set_token};
use state::{AppConfig, AppState};

fn main() -> Result<(), MainError> {
    let matches = clap::App::new(env!("CARGO_PKG_NAME"))
        .version(env!("CARGO_PKG_VERSION"))
        .author(env!("CARGO_PKG_AUTHORS"))
        .about(env!("CARGO_PKG_DESCRIPTION"))
        .arg(
            clap::Arg::with_name("host")
                .long("host")
                .short("h")
                .value_name("HOSTNAME")
                .default_value("0.0.0.0")
                .help("Sets a hostname for HTTP-server")
                .takes_value(true)
                .display_order(1),
        )
        .arg(
            clap::Arg::with_name("port")
                .long("port")
                .short("p")
                .value_name("PORT")
                .default_value("8080")
                .help("Sets a port for HTTP-server")
                .takes_value(true)
                .display_order(2),
        )
        .arg(
            clap::Arg::with_name("redis-host")
                .long("redis-host")
                .short("H")
                .value_name("HOSTNAME")
                .default_value("localhost")
                .help("Sets a hostname for redis connection")
                .takes_value(true)
                .display_order(3),
        )
        .arg(
            clap::Arg::with_name("redis-port")
                .long("redis-port")
                .short("P")
                .value_name("PORT")
                .default_value("6379")
                .help("Sets a port for redis connection")
                .takes_value(true)
                .required(false)
                .display_order(4),
        )
        .arg(
            clap::Arg::with_name("db-index")
                .long("db-index")
                .short("D")
                .value_name("INDEX")
                .default_value("0")
                .help("Selects a redis db")
                .takes_value(true)
                .display_order(5),
        )
        .arg(
            clap::Arg::with_name("redis-password")
                .long("redis-password")
                .short("W")
                .value_name("PASSWORD")
                .help("Sets a redis password. Also you can to use $PASSWORD environment variable")
                .takes_value(true)
                .display_order(6),
        )
        .arg(
            clap::Arg::with_name("header")
                .long("header")
                .short("d")
                .help("Sets a header")
                .takes_value(true)
                .required(false)
                .value_names(&["NAME", "VALUE"])
                .value_delimiter(":")
                .require_delimiter(true)
                .multiple(true)
                .display_order(7),
        )
        .get_matches();

    let app_config = AppConfig {
        host: matches
            .value_of("host")
            .map(String::from)
            .unwrap_or_default(),

        port: matches
            .value_of("port")
            .map(String::from)
            .unwrap_or_default(),

        redis_host: matches
            .value_of("redis-host")
            .map(String::from)
            .unwrap_or_default(),

        redis_port: matches
            .value_of("redis-port")
            .unwrap_or_default()
            .parse::<u16>()?,

        db_index: matches
            .value_of("db-index")
            .unwrap_or_default()
            .parse::<i64>()?,

        redis_password: matches
            .value_of("redis-password")
            .map(String::from)
            .or(var("PASSWORD").ok()),
    };

    let AppConfig {
        host,
        port,
        db_index: db,
        redis_password: passwd,
        redis_host,
        redis_port,
    } = app_config.clone();

    let addr = Box::new(ConnectionAddr::Tcp(redis_host, redis_port));

    let client = Client::open(ConnectionInfo { addr, db, passwd })?;

    let connection = client.get_connection()?;

    let client_mutex = Arc::new(Mutex::new(client));
    let connection_mutex = Arc::new(Mutex::new(connection));
    let config_mutex = Arc::new(Mutex::new(app_config));

    server::new(move || {
        matches
            .values_of("header")
            .map(|headers| -> Vec<&str> {
                let headers: Vec<_> = headers.collect();
                headers
            })
            .unwrap_or(Vec::new())
            .chunks(2)
            .map(|header| {
                let name = HeaderName::from_bytes(header[0].as_bytes()).unwrap();
                let value = HeaderValue::from_bytes(header[1].as_bytes()).unwrap();
                middleware::DefaultHeaders::new().header(name, value)
            })
            .fold(
                App::with_state(AppState::new(
                    &client_mutex,
                    &connection_mutex,
                    &config_mutex,
                )),
                |app, mw| app.middleware(mw),
            )
            .middleware(
                middleware::DefaultHeaders::new().header("Content-type", "application/json"),
            )
            .resource("/tokens", |resource| {
                resource.method(Method::POST).f(set_token)
            })
            .resource("/tokens/{value}", |resource| {
                resource.method(Method::GET).f(get_token)
            })
            .resource("/tokens/{value}", |resource| {
                resource.method(Method::DELETE).f(delete_token)
            })
    })
    .bind(format!("{}:{}", host, port))
    .unwrap()
    .run();

    Ok(())
}
