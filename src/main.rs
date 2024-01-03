#![warn(clippy::all)]

mod profanity;
mod routes;
mod store;
mod types;
mod utils;

use std::env;

use routes::{
    answers::add_a,
    questions::{add_q, del_q, detail_q, get_q, upd_q},
};
use store::Store;

use tracing::info;
use tracing::Span;
use tracing_subscriber::fmt::format::FmtSpan;
use uuid::Uuid;
use warp::{
    reject::Reject,
    trace::{Info, Trace},
    Filter,
};

use crate::routes::auth::{login, register};
use config::Config;

#[derive(Debug, serde::Deserialize)]
struct AppConfig {
    log_level: String,
    database_host: String,
    database_port: u16,
    database_name: String,
    port: u16,
}

#[tokio::main]
async fn main() -> Result<(), sqlx::Error> {
  println!("{:?}", env::args());
  
    let conf = Config::builder()
        .add_source(config::File::with_name("config"))
        .build()
        .unwrap();

    let app_config = conf.try_deserialize::<AppConfig>().unwrap();

    setup_app(app_config).await;
    Ok(())
}

async fn setup_app(conf: AppConfig) {
    tracing_subscriber::fmt()
        .with_env_filter(format!(
            "helloworld={},warp={},error-handler={},sqlx={}",
            conf.log_level, conf.log_level, conf.log_level, conf.log_level
        ))
        .with_span_events(FmtSpan::CLOSE)
        .init();

    let db_url = env::var("DB_URL").unwrap_or(format!(
        "postgres://dev:dev@{}:{}/{}",
        conf.database_host, conf.database_port, conf.database_name
    ));
    info!("DB connection url: {}", db_url);

    let store = Store::new(&db_url).await;

    info!("Start db migration");
    sqlx::migrate!()
        .run(&store.clone().pool)
        .await
        .expect("Could not run db migration");

    info!("Finish db migration");

    let store_filter = warp::any().map(move || store.clone());

    let get_q = warp::get()
        .and(warp::path("q"))
        .and(warp::path::end())
        .and(warp::query())
        .and(store_filter.clone())
        .and_then(get_q);

    let add_q = warp::post()
        .and(warp::path("q"))
        .and(warp::path::end())
        .and(routes::auth::auth())
        .and(store_filter.clone())
        .and(warp::body::json())
        .and_then(add_q);

    let detail_q = warp::get()
        .and(warp::path("q"))
        .and(warp::path::param::<u32>())
        .and(warp::path::end())
        .and(store_filter.clone())
        .and_then(detail_q);

    let upd_q = warp::put()
        .and(warp::path("q"))
        .and(warp::path::param::<u32>())
        .and(warp::path::end())
        .and(store_filter.clone())
        .and(warp::body::json())
        .and_then(upd_q);

    let del_q = warp::delete()
        .and(warp::path("q"))
        .and(warp::path::param::<u32>())
        .and(warp::path::end())
        .and(store_filter.clone())
        .and_then(del_q);

    let add_a = warp::post()
        .and(warp::path("a"))
        .and(warp::path::end())
        .and(store_filter.clone())
        .and(warp::body::form())
        .and_then(add_a);

    let register = warp::post()
        .and(warp::path("reg"))
        .and(warp::path::end())
        .and(store_filter.clone())
        .and(warp::body::json())
        .and_then(register);

    let login = warp::post()
        .and(warp::path("login"))
        .and(warp::path::end())
        .and(store_filter.clone())
        .and(warp::body::json())
        .and_then(login);

    let routes = get_q
        .or(add_q)
        .or(detail_q)
        .or(upd_q)
        .or(del_q)
        .or(add_a)
        // .with(log)
        .or(register)
        .or(login)
        .with(cors_conf())
        .with(trace_conf())
        .with(warp::trace::request())
        .recover(error_handler::error_hanling);

    warp::serve(routes).run(([0, 0, 0, 0], conf.port)).await
}

fn trace_conf() -> Trace<impl Fn(Info<'_>) -> Span + Clone> {
    warp::trace(|info| {
        let id = match info.request_headers().get("id") {
            Some(id) => String::from(id.to_str().unwrap_or("default")),
            None => Uuid::new_v4().to_string(),
        };
        tracing::info_span!(
            "ID",
            id = %id
        )
    })
}

fn cors_conf() -> warp::cors::Builder {
    warp::cors()
        .allow_any_origin()
        .allow_header("content-type")
        .allow_methods(&[
            warp::http::Method::PUT,
            warp::http::Method::DELETE,
            warp::http::Method::GET,
            warp::http::Method::POST,
        ])
}

#[derive(Debug)]
struct InvalidId;

impl Reject for InvalidId {}
