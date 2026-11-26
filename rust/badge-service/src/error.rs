use std::fmt;
use std::io;
use std::num::ParseIntError;
use std::time::SystemTimeError;

use base64::DecodeError;
use serde_json::Error as JsonError;

#[derive(Debug)]
pub enum ApiError {
    BadRequest(String),
    Forbidden(String),
    NotFound,
    Internal(String),
}

impl ApiError {
    pub fn status(&self) -> u16 {
        match self {
            ApiError::BadRequest(_) => 400,
            ApiError::Forbidden(_) => 403,
            ApiError::NotFound => 404,
            ApiError::Internal(_) => 500,
        }
    }

    pub fn message(&self) -> String {
        match self {
            ApiError::BadRequest(m) | ApiError::Forbidden(m) | ApiError::Internal(m) => m.clone(),
            ApiError::NotFound => "not found".to_string(),
        }
    }
}

impl fmt::Display for ApiError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.message())
    }
}

impl std::error::Error for ApiError {}

impl From<io::Error> for ApiError {
    fn from(e: io::Error) -> Self {
        ApiError::Internal(e.to_string())
    }
}

impl From<JsonError> for ApiError {
    fn from(e: JsonError) -> Self {
        ApiError::BadRequest(e.to_string())
    }
}

impl From<DecodeError> for ApiError {
    fn from(e: DecodeError) -> Self {
        ApiError::BadRequest(e.to_string())
    }
}

impl From<ParseIntError> for ApiError {
    fn from(e: ParseIntError) -> Self {
        ApiError::BadRequest(e.to_string())
    }
}

impl From<SystemTimeError> for ApiError {
    fn from(e: SystemTimeError) -> Self {
        ApiError::Internal(e.to_string())
    }
}

pub fn bad_request<S: Into<String>>(message: S) -> ApiError {
    ApiError::BadRequest(message.into())
}

pub fn forbidden<S: Into<String>>(message: S) -> ApiError {
    ApiError::Forbidden(message.into())
}

pub fn internal<S: Into<String>>(message: S) -> ApiError {
    ApiError::Internal(message.into())
}
