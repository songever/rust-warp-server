#![warn(clippy::all)]

use clap::Parser;
use dotenv;
use handle_errors::return_error;
use std::env;
use tracing_subscriber::fmt::format::FmtSpan;
use warp::{Filter, http::Method};



mod profanity;
mod routes;
mod store;
mod types;
mod config;

#[derive(Parser, Debug)]
#[clap(author, version, about, long_about = None)]
struct Args {
    #[clap(short, long, default_value = "warn")]
    log_level: String,
    ///Which PORT the server is listening to
    #[clap(short, long, default_value = "8080")]
    port: u16,
    ///Database user
    #[clap(long, default_value = "root")]
    db_user: String,
    #[clap(long, default_value = "root:Mei20031003@localhost")]
    db_host: String,
    #[clap(long, default_value = "5432")]
    db_port: u16,
    #[clap(long, default_value = "rustwebdev")]
    db_name: String,
}

#[tokio::main]
async fn main() -> Result<(), handle_errors::Error> {
    dotenv::dotenv().ok();

    let config = config::Config::new().expect("Config can't be set");
    if let Err(_) = env::var("BAD_WORDS_API_KEY") {
        panic!("Badwords API key not set");
    }

    if let Err(_) = env::var("PASETO_KEY") {
        panic!("PASETO key not set");
    }

    let args = Args::parse();

    let port = std::env::var("PORT")
        .ok()
        .map(|val| val.parse::<u16>())
        .unwrap_or(Ok(3030))
        .map_err(|e| handle_errors::Error::ParseError(e));

    let db_host = std::env::var("POSTGRES_HOST")
        .unwrap_or(args.db_host.to_owned());
    let db_password = std::env::var("POSTGRES_PASSWORD").unwrap();
    let db_user = std::env::var("POSTGRES_USER")
        .unwrap_or(args.db_user.to_owned());
    let db_port = std::env::var("POSTGRES_PORT")
        .unwrap_or(args.db_port.to_string());
    let db_name = std::env::var("POSTGRES_DB")
        .unwrap_or(args.db_name.to_owned());

    let log_filter = std::env::var("RUST_LOG").unwrap_or_else(|_| {
        format!(
            "handle_errors={},rust_web_dev={},warp={}",
            args.log_level, args.log_level, args.log_level
        )
    });

    let store = store::Store::new(&format!(
        "postgres://{}:{}@{}:{}/{}",
        db_user, db_password,
        db_host, db_port, db_name
    ))
    .await
    .map_err(|e| handle_errors::Error::DatabaseQueryError(e))?;

    sqlx::migrate!("./migrations")
        .run(&store.clone().connection)
        .await
        .map_err(|e| handle_errors::Error::MigrationError(e))?;

    let store_filter = warp::any().map(move || store.clone());

    tracing_subscriber::fmt()
        // Use the filter we built above to determine which traces to record.
        .with_env_filter(log_filter)
        // Record an event when each span closes. This can be used to time our
        // routes' durations!
        .with_span_events(FmtSpan::CLOSE)
        .with_level(true)
        .init();

    let cors = warp::cors()
        .allow_any_origin()
        .allow_header("content-type")
        .allow_methods(&[Method::PUT, Method::DELETE, Method::GET, Method::POST]);

    let get_questions = warp::get()
        .and(warp::path("questions"))
        .and(warp::path::end())
        .and(warp::query())
        .and(store_filter.clone())
        .and_then(routes::question::get_questions);

    let add_question = warp::post()
        .and(warp::path("questions"))
        .and(warp::path::end())
        .and(routes::authentication::auth())
        .and(store_filter.clone())
        .and(warp::body::json())
        .and_then(routes::question::add_question);

    let update_question = warp::put()
        .and(warp::path("questions"))
        .and(warp::path::param::<i32>())
        .and(warp::path::end())
        .and(routes::authentication::auth())
        .and(store_filter.clone())
        .and(warp::body::json())
        .and_then(routes::question::update_question);

    let delete_question = warp::delete()
        .and(warp::path("questions"))
        .and(warp::path::param::<i32>())
        .and(warp::path::end())
        .and(routes::authentication::auth())
        .and(store_filter.clone())
        .and_then(routes::question::delete_question);

    let add_answer = warp::post()
        .and(warp::path("answers"))
        .and(warp::path::end())
        .and(routes::authentication::auth())
        .and(store_filter.clone())
        .and(warp::body::form())
        .and_then(routes::answer::add_answer);

    let registration = warp::post()
        .and(warp::path("registration"))
        .and(warp::path::end())
        .and(store_filter.clone())
        .and(warp::body::json())
        .and_then(routes::authentication::register);

    let login = warp::post()
        .and(warp::path("login"))
        .and(warp::path::end())
        .and(store_filter.clone())
        .and(warp::body::json())
        .and_then(routes::authentication::login);

    let routes = update_question
        .or(delete_question)
        .or(add_question)
        .or(add_answer)
        .or(get_questions)
        .or(registration)
        .or(login)
        .with(cors)
        .with(warp::trace::request())
        .recover(return_error);

    tracing::info!("Q&A service build ID {}", env!("RUST_WEB_DEV_VERSION"));

    warp::serve(routes)
        .run(([0, 0, 0, 0], port.unwrap()))
        .await;

    Ok(())
}
