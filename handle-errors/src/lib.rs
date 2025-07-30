use warp::{
    filters::{
        body::BodyDeserializeError,
        cors::CorsForbidden,
    }, 
    reject::Reject,
    Rejection, 
    Reply, 
    http::StatusCode,
};
use tracing::{event, Level, instrument};
use reqwest::Error as ReqwestError;
use reqwest_middleware::Error as MiddlewareReqwestError;
#[derive(Debug)]
pub enum Error {
    ParseError(std::num::ParseIntError),
    MissingParameters,
    WrongPassword,
    CannotDecryptToken,
    Unauthorized,
    ArgonLibraryError(argon2::Error),
    QuestionNotFound,
    DatabaseQueryError(sqlx::Error),
    MigrationError(sqlx::migrate::MigrateError),
    ReqwestAPIError(ReqwestError),
    MiddlewareReqwestError(MiddlewareReqwestError),
    ClientError(APILayerError),
    ServerError(APILayerError),
}
#[derive(Debug)]
pub struct APILayerError {
    pub status: u16,
    pub message: String,
}

impl std::fmt::Display for APILayerError {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(f, "Status: {}, Message: {}", self.status, self.message)
    }
}
impl std::fmt::Display for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        match self {
            Error::ParseError(err) => {
                write!(f, "Cannot parse parameter: {}", err)
            }
            Error::MissingParameters => write!(f, "Missing parameter"),
            Error::WrongPassword => write!(f, "Wrong password"),
            Error::CannotDecryptToken => {
                write!(f, "Cannot decrypt error")
            }
            Error::Unauthorized => write!(f, 
                "No permisssion to change the underlying rsource"),
            Error::ArgonLibraryError(_) => {
                write!(f, "Connot verify password")
            }
            Error::QuestionNotFound => write!(f, "Question not found"),
            Error::DatabaseQueryError(_) => {
                write!(f, "Cannot update, invalid data.")
            }
            Error::MigrationError(_) => write!(f, "Cannot migrate data"),
            Error::ReqwestAPIError(err) => {
                write!(f, "External API Error: {}", err)
            }
            Error::MiddlewareReqwestError(err) => {
                write!(f, "External API Error: {}", err)
            }
            Error::ClientError(err) => {
                write!(f, "External Client error: {}", err)
            }
            Error::ServerError(err) => {
                write!(f, "External Server error: {}", err)
            }
        }
    }
}
const DUPLICATE_KEY: u32 = 23505;

impl Reject for Error {}

#[instrument]
pub async fn return_error(r: Rejection) -> Result<impl Reply, Rejection> {
    println!("{:?}", r);
    if let Some(crate::Error::DatabaseQueryError(e)) = r.find() {
        event!(Level::ERROR, "Database query error");

        match e {
            sqlx::Error::Database(err) => {
                if err.code().unwrap().parse::<u32>().unwrap() == DUPLICATE_KEY {
                    Ok(warp::reply::with_status(
                            "Account already exists".to_string(),
                            StatusCode::UNPROCESSABLE_ENTITY,
                        ))
                } else {
                    Ok(warp::reply::with_status(
                        "Cannot update data".to_string(),
                        StatusCode::UNPROCESSABLE_ENTITY,
                    ))
                }
            }
            _ => {
                Ok(warp::reply::with_status(
                    "Cannot not update data".to_string(),
                    StatusCode::UNPROCESSABLE_ENTITY,
                ))  
            }
        }
        
    } else if let Some(crate::Error::ReqwestAPIError(e)) = r.find() {
        event!(Level::ERROR, "{}", e);
        Ok(warp::reply::with_status(
            "Internal Server Error".to_string(),
            StatusCode::INTERNAL_SERVER_ERROR, 
        ))
    } else if let Some(crate::Error::Unauthorized) = r.find() {
        event!(Level::ERROR, "Not matching account id");
        Ok(warp::reply::with_status(
            "No permission to changing underlying resource".to_string(),
            StatusCode::UNAUTHORIZED, 
        ))
    } else if let Some(crate::Error::WrongPassword) = r.find() {
        event!(Level::ERROR, "Entered wrong password");
        Ok(warp::reply::with_status(
            "Wrong E-Mail/Password combination".to_string(),
            StatusCode::UNAUTHORIZED, 
        ))
    } else if let Some(crate::Error::MiddlewareReqwestError(e)) = r.find() {
        event!(Level::ERROR, "{}", e);
        Ok(warp::reply::with_status(
            "Internal Server Error".to_string(),
            StatusCode::INTERNAL_SERVER_ERROR, 
        ))
    } else if let Some(crate::Error::ClientError(e)) = r.find() {
        event!(Level::ERROR, "{}", e);
        Ok(warp::reply::with_status(
            "Internal Server Error".to_string(),
            StatusCode::INTERNAL_SERVER_ERROR, 
        ))
    } else if let Some(crate::Error::ServerError(e)) = r.find() {
        event!(Level::ERROR, "{}", e);
        Ok(warp::reply::with_status(
            "Internal Server Error".to_string(),
            StatusCode::INTERNAL_SERVER_ERROR, 
        ))
    } else if let Some(error) = r.find::<BodyDeserializeError>() {
        event!(Level::ERROR, "Cannot deserizalize request body: {}", error);
        Ok(warp::reply::with_status(
            error.to_string(),
            StatusCode::UNPROCESSABLE_ENTITY, 
        ))
    } else if let Some(error) = r.find::<Error>() {
        event!(Level::ERROR, "{}", error);
        Ok(warp::reply::with_status(
            error.to_string(),
            StatusCode::RANGE_NOT_SATISFIABLE,
        ))
    } else if let Some(error) = r.find::<CorsForbidden>() {
        event!(Level::ERROR, "CORS forbidden error: {}", error);
        Ok(warp::reply::with_status(
            error.to_string(),
            StatusCode::FORBIDDEN,
        ))
    } else {
        event!(Level::WARN, "Requested route was not found");
        Ok(warp::reply::with_status(
            "Route not found".to_string(),
            StatusCode::NOT_FOUND, 
        ))
    }
}