use std::convert::From;
use std::env::VarError;
use std::fmt;
use std::num::ParseIntError;
use std::string::ParseError;
use std::time::SystemTimeError;

use failure::Fail;

#[allow(unused_imports)]
use serde::Serializer;
use serde_derive::{Deserialize, Serialize};

use actix_web::error::{InternalError, JsonPayloadError, ResponseError};
use actix_web::http::header;
use actix_web::HttpResponse;

use redis::RedisError;

#[derive(Debug, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct MainErrorBody {
    pub error_type: String,
    pub description: String,
}

impl From<&MainError> for MainErrorBody {
    fn from(error: &MainError) -> Self {
        MainErrorBody {
            error_type: String::from(match error {
                MainError::RedisError(_) => "RedisError",
                MainError::VarError(_) => "VarError",
                MainError::ParseIntError(_) => "ParseIntError",
                MainError::SystemTimeError(_) => "SystemTimeError",
                MainError::ParamsParseError(_) => "ParamsParseError",
                MainError::ActixError(_) => "ActixError",
                MainError::JsonPayloadError(_) => "JsonPayloadError",
            }),
            description: format!("{}", error),
        }
    }
}

#[derive(Fail, Debug)]
pub enum MainError {
    RedisError(RedisError),
    VarError(VarError),
    ParseIntError(ParseIntError),
    SystemTimeError(SystemTimeError),
    ParamsParseError(InternalError<ParseError>),
    ActixError(actix_web::error::Error),
    JsonPayloadError(JsonPayloadError),
}

impl ResponseError for MainError {
    fn error_response(&self) -> HttpResponse {
        match self {
            MainError::RedisError(_)
            | MainError::VarError(_)
            | MainError::ParseIntError(_)
            | MainError::SystemTimeError(_)
            | MainError::ParamsParseError(_) => HttpResponse::InternalServerError(),
            MainError::ActixError(_) => HttpResponse::NotFound(),
            MainError::JsonPayloadError(_) => HttpResponse::BadRequest(),
        }
        .header(header::CONTENT_TYPE, "application/json")
        .body(serde_json::to_string(&MainErrorBody::from(self)).unwrap())
    }
}

impl fmt::Display for MainError {
    fn fmt(&self, f: &mut fmt::Formatter) -> Result<(), fmt::Error> {
        match self {
            MainError::RedisError(err) => err.fmt(f),
            MainError::VarError(err) => err.fmt(f),
            MainError::ParseIntError(err) => err.fmt(f),
            MainError::SystemTimeError(err) => err.fmt(f),
            MainError::ParamsParseError(err) => err.fmt(f),
            MainError::ActixError(err) => err.fmt(f),
            MainError::JsonPayloadError(err) => err.fmt(f),
        }
    }
}

impl Clone for MainError {
    #[allow(unconditional_recursion)]
    fn clone(&self) -> MainError {
        Clone::clone(self)
    }
}

impl From<&MainError> for MainError {
    fn from(error: &MainError) -> Self {
        match error.clone() {
            MainError::RedisError(err) => MainError::RedisError(err),
            MainError::VarError(err) => MainError::VarError(err),
            MainError::ParseIntError(err) => MainError::ParseIntError(err),
            MainError::SystemTimeError(err) => MainError::SystemTimeError(err),
            MainError::ParamsParseError(err) => MainError::ParamsParseError(err),
            MainError::ActixError(err) => MainError::ActixError(err),
            MainError::JsonPayloadError(err) => MainError::JsonPayloadError(err),
        }
    }
}

impl From<VarError> for MainError {
    fn from(error: VarError) -> Self {
        MainError::VarError(error)
    }
}

impl From<RedisError> for MainError {
    fn from(error: RedisError) -> Self {
        MainError::RedisError(error)
    }
}

impl From<ParseIntError> for MainError {
    fn from(error: ParseIntError) -> Self {
        MainError::ParseIntError(error)
    }
}

impl From<SystemTimeError> for MainError {
    fn from(error: SystemTimeError) -> Self {
        MainError::SystemTimeError(error)
    }
}

impl From<InternalError<ParseError>> for MainError {
    fn from(error: InternalError<ParseError>) -> Self {
        MainError::ParamsParseError(error)
    }
}

impl From<actix_web::error::Error> for MainError {
    fn from(error: actix_web::error::Error) -> Self {
        MainError::ActixError(error)
    }
}

impl From<JsonPayloadError> for MainError {
    fn from(error: JsonPayloadError) -> Self {
        MainError::JsonPayloadError(error)
    }
}
