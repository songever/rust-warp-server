#![warn(clippy::all)]

use clap::Parser;
use dotenv;
use handle_errors::return_error;
use tracing_subscriber::fmt::format::FmtSpan;
use warp::{Filter, http::Method};
use tokio::sync::{oneshot, oneshot::Sender};
pub use handle_errors;

mod profanity;
mod routes;
mod store;
mod types;
pub mod config;

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

pub struct OneshotHandler {
    pub sender: Sender<i32>,
}

async fn build_routes(store: store::Store) -> 
    impl Filter<Extract = impl warp::Reply, Error = warp::Rejection> + Clone {
    let store_filter = warp::any().map(move || store.clone());

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

    update_question
        .or(delete_question)
        .or(add_question)
        .or(add_answer)
        .or(get_questions)
        .or(registration)
        .or(login)
        .with(cors)
        .with(warp::trace::request())
        .recover(return_error)
}

pub async fn setup_store(
    config: &config::Config,
) -> Result<store::Store, handle_errors::Error> {
    dotenv::dotenv().ok();

    // let config = config::Config::new().expect("Config can't be set");
    // if let Err(_) = env::var("BAD_WORDS_API_KEY") {
    //     panic!("Badwords API key not set");
    // }

    // if let Err(_) = env::var("PASETO_KEY") {
    //     panic!("PASETO key not set");
    // }

    // let args = Args::parse();

    // let port = std::env::var("PORT")
    //     .ok()
    //     .map(|val| val.parse::<u16>())
    //     .unwrap_or(Ok(3030))
    //     .map_err(|e| handle_errors::Error::ParseError(e));

    // let db_host = std::env::var("POSTGRES_HOST")
    //     .unwrap_or(args.db_host.to_owned());
    // let db_password = std::env::var("POSTGRES_PASSWORD").unwrap();
    // let db_user = std::env::var("POSTGRES_USER")
    //     .unwrap_or(args.db_user.to_owned());
    // let db_port = std::env::var("POSTGRES_PORT")
    //     .unwrap_or(args.db_port.to_string());
    // let db_name = std::env::var("POSTGRES_DB")
    //     .unwrap_or(args.db_name.to_owned());

    let store = store::Store::new(&format!(
        "postgres://{}:{}@{}:{}/{}",
        config.db_user, config.db_password,
        config.db_host, config.db_port, config.db_name
    ))
    .await
    .map_err(|e| handle_errors::Error::DatabaseQueryError(e))?;

    sqlx::migrate!("./migrations")
        .run(&store.clone().connection)
        .await
        .map_err(|e| handle_errors::Error::MigrationError(e))?;

    let log_filter = std::env::var("RUST_LOG").unwrap_or_else(|_| {
            format!(
                "handle_errors={},rust_web_dev={},warp={}",
                config.log_level, config.log_level, config.log_level
            )
        });

    tracing_subscriber::fmt()
    // Use the filter we built above to determine which traces to record.
    .with_env_filter(log_filter)
    // Record an event when each span closes. This can be used to time our
    // routes' durations!
    .with_span_events(FmtSpan::CLOSE)
    .with_level(true)
    .init();

    Ok(store)
}

pub async fn run(config: config::Config, store: store::Store) {
    let routes = build_routes(store).await;

    warp::serve(routes)
        .run(([0, 0, 0, 0], config.port))
        .await;
}

pub async fn oneshot(store: store::Store) -> OneshotHandler {
    let routes = build_routes(store).await;
    let (tx, rx) = oneshot::channel::<i32>();

    let socket: std::net::SocketAddr = "127.0.0.1:3030"
        .to_string()
        .parse()
        .expect("Not a valid socket address");

    let (_, server) = warp::serve(routes).bind_with_graceful_shutdown(
        socket, async {
        rx.await.ok();
    });

    tokio::spawn(server);

    OneshotHandler {sender: tx}
}
