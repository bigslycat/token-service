use futures::Future;

use actix_web::error::ErrorNotFound;
use actix_web::{AsyncResponder, FutureResponse, HttpMessage, HttpRequest, HttpResponse, Json};

use redis::cmd;

use error::MainError;
use helpers::now;
use state::AppState;
use token::{generate, TokenData, TokenPayload};

pub fn set_token(req: &HttpRequest<AppState>) -> FutureResponse<HttpResponse, MainError> {
    let connection_mutex = req.state().get_connection();
    req.json()
        .from_err()
        .and_then(
            move |data: TokenPayload| -> Result<HttpResponse, MainError> {
                let connection = &*connection_mutex.lock().unwrap();

                let TokenPayload {
                    value,
                    user,
                    expires,
                } = data;

                let value = value.unwrap_or_else(generate);

                cmd("SET").arg(&value).arg(&user).query(connection)?;

                let mut set = cmd("SET");
                let query = set.arg(&value).arg(&user);

                match expires {
                    Some(expires) => query.arg("EX").arg(expires - now()),
                    None => query,
                }
                .query(connection)?;

                Ok(HttpResponse::Ok().json(TokenData {
                    value,
                    user,
                    expires,
                }))
            },
        )
        .responder()
}

pub fn get_token(req: &HttpRequest<AppState>) -> Result<Json<TokenData>, MainError> {
    let connection_mutex = req.state().get_connection();
    let connection = &*connection_mutex.lock().unwrap();

    let value = req.match_info().query::<String>("value")?.clone();

    let user = redis::cmd("GET")
        .arg(&value)
        .query::<Option<String>>(connection)?
        .ok_or(ErrorNotFound(format!("Token {} not found", value)))?;

    let expires = redis::cmd("TTL")
        .arg(&value)
        .query::<Option<i64>>(connection)?
        .filter(|ttl| !ttl.is_negative())
        .map(|ttl| ttl as u64 + now());

    Ok(Json(TokenData {
        value,
        user,
        expires,
    }))
}

pub fn delete_token(req: &HttpRequest<AppState>) -> Result<HttpResponse, MainError> {
    let connection_mutex = req.state().get_connection();
    let connection = &*connection_mutex.lock().unwrap();

    let value = req.match_info().query::<String>("value")?;

    redis::cmd("DEL").arg(&value).query(connection)?;

    Ok(HttpResponse::Ok().finish())
}
