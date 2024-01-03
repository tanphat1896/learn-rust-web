use std::fmt::Display;
use std::num::ParseIntError;

use tracing::instrument;
use warp::body::BodyDeserializeError;
use warp::cors::CorsForbidden;
use warp::reject::Reject;
use warp::{http::StatusCode, reply, Rejection, Reply};

#[derive(Debug)]
pub enum AppError {
  ParseError(ParseIntError),
  MissingParams,
  InvalidRange,
  QuestionNotFound,
  InconsistenceId,
  DbError,
  DbQueryError,
  ApiCallErr(String),
  InvalidCredential,
  InvalidToken
}

impl Display for AppError {
  fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
    match self {
      AppError::ParseError(_e) => write!(f, "Cannot parse param: {}", _e),
      AppError::MissingParams => write!(f, "Missing required param"),
      AppError::InvalidRange => write!(f, "Invalid range"),
      AppError::QuestionNotFound => write!(f, "Question not found"),
      AppError::InconsistenceId => write!(f, "Question ID mismatched"),
      AppError::DbError => write!(f, "DB error"),
      AppError::DbQueryError => write!(f, "DB access failed"),
      AppError::ApiCallErr(reason) => write!(f, "External api call got error {}", reason),
      AppError::InvalidCredential => write!(f, "Invalid login credentials"),
      AppError::InvalidToken => write!(f, "Unauthorized token")
    }
  }
}

impl Reject for AppError {}

/**
 * Handle and return http error.
 * The default error is 406
 */
#[instrument]
pub async fn error_hanling(r: Rejection) -> Result<impl Reply, Rejection> {
  println!("{:?}", r);

  if let Some(AppError::InvalidToken) = r.find() {
    return Ok(reply::with_status(
      AppError::InvalidToken.to_string(),
      StatusCode::UNAUTHORIZED,
    ));
  }

  if let Some(AppError::DbQueryError) = r.find() {
    return Ok(reply::with_status(
      AppError::DbQueryError.to_string(),
      StatusCode::INTERNAL_SERVER_ERROR,
    ));
  }

  if let Some(AppError::ApiCallErr(str)) = r.find() {
    return Ok(reply::with_status(
      str.to_string(),
      StatusCode::INTERNAL_SERVER_ERROR,
    ));
  }

  if let Some(e) = r.find::<BodyDeserializeError>() {
    return Ok(reply::with_status(
      e.to_string(),
      StatusCode::UNPROCESSABLE_ENTITY,
    ));
  }
  if let Some(e) = r.find::<AppError>() {
    return Ok(reply::with_status(
      e.to_string(),
      StatusCode::RANGE_NOT_SATISFIABLE,
    ));
  }
  if let Some(e) = r.find::<CorsForbidden>() {
    return Ok(reply::with_status(e.to_string(), StatusCode::FORBIDDEN));
  }
  Ok(reply::with_status(
    "Resource not found".to_string(),
    StatusCode::NOT_ACCEPTABLE,
  ))
}
