use std::sync::Arc;
use axum::{
    extract::{Path, Query, State},
    http::{header, HeaderMap, HeaderValue, StatusCode},
    response::{IntoResponse, Response},
    Json,
};
use mongodb::bson::oid::ObjectId;

use crate::{
    error::MyError, token::{self, verify_jwt_token, TokenDetails}, user::{
        model::TokenClaims, response::{MessageResponse, TokenMessageResponse}, schema::{CreateUserSchema, FilterOptions, UpdateUserSchema, VaildUserSchema}
    }, AppState
};
use jsonwebtoken::{decode, encode, Algorithm, EncodingKey, Header};
use chrono::{DateTime, Utc};
use axum_extra::extract::cookie::{Cookie, SameSite, CookieJar};
use argon2::{password_hash::SaltString, Argon2, PasswordHash, PasswordHasher, PasswordVerifier};
use rand_core::OsRng;
use serde_json::json;
use redis::AsyncCommands;

fn generate_token(
    id: Option<ObjectId>,
    max_age: i64,
    private_key: String,
) -> Result<TokenDetails, (StatusCode, Json<serde_json::Value>)> {
    token::generate_jwt_token(id, max_age, private_key).map_err(|e| {
        let error_response = serde_json::json!({
            "status": "error",
            "message": format!("error generating token: {}", e),
        });
        (StatusCode::INTERNAL_SERVER_ERROR, Json(error_response))
    })
}

async fn save_token_data_to_redis(
    data: &Arc<AppState>,
    token_details: &TokenDetails,
    max_age: i64,
) -> Result<(), (StatusCode, Json<serde_json::Value>)> {
    let mut redis_client: redis::aio::Connection = data
        .redis_client
        .get_async_connection()
        .await
        .map_err(|e| {
            let error_response = serde_json::json!({
                "status": "error",
                "message": format!("Redis error: {}", e),
            });
            (StatusCode::INTERNAL_SERVER_ERROR, Json(error_response))
        })?;
    redis_client
        .set_ex(
            token_details.token_uuid.to_string(),
            token_details.id.unwrap().to_string(),
            (max_age * 60) as u64,
        )
        .await
        .map_err(|e| {
            let error_response = serde_json::json!({
                "status": "error",
                "message": format_args!("{}", e),
            });
            (StatusCode::UNPROCESSABLE_ENTITY, Json(error_response))
        })?;
    Ok(())
}

fn verify_token_handler(
    token: Option<&HeaderValue>,
    State(app_state): State<Arc<AppState>>
)-> Result<impl IntoResponse, (StatusCode, Json<serde_json::Value>)> {
    if(token.is_none()){
        let error_response = serde_json::json!({
            "status": "fail",
            "message": "Token is empty"
        });
        return Err((StatusCode::UNAUTHORIZED, Json(error_response)));
    }

    let tokenstr = token.unwrap().to_str().unwrap();
    match token::verify_jwt_token(app_state.env.access_token_public_key.to_owned(), &tokenstr)
    {
        Ok(token_details) => token_details,
        Err(e) => {
            let error_response = serde_json::json!({
                "status": "fail",
                "message": format_args!("{:?}", e)
            });
            return Err((StatusCode::UNAUTHORIZED, Json(error_response)));
        }
    };

    return Ok(StatusCode::OK);
}

pub async fn user_list_handler(
    opts: Option<Query<FilterOptions>>,
    headers: HeaderMap,
    State(app_state): State<Arc<AppState>>,
) -> Result<impl IntoResponse, (StatusCode, Json<serde_json::Value>)> {
    let token = headers.get("token");
    if(token.is_none()){
        let error_response = serde_json::json!({
            "status": "fail",
            "message": "Token is empty"
        });
        return Err((StatusCode::UNAUTHORIZED, Json(error_response)));
    }

    let tokenstr = token.unwrap().to_str().unwrap();
    match token::verify_jwt_token(app_state.env.access_token_public_key.to_owned(), &tokenstr)
    {
        Ok(token_details) => token_details,
        Err(e) => {
            let error_response = serde_json::json!({
                "status": "fail",
                "message": format_args!("{:?}", e)
            });
            return Err((StatusCode::UNAUTHORIZED, Json(error_response)));
        }
    };

    //dbg!(token_details);
    //verify_token_handler(token, app_state);

    let Query(opts) = opts.unwrap_or_default();

    let limit = opts.limit.unwrap_or(10) as i64;
    let page = opts.page.unwrap_or(1) as i64;

    match app_state
        .db
        .fetch_users(limit, page)
        .await
        .map_err(MyError::from)
    {
        Ok(res) => Ok(Json(res)),
        Err(e) => Err(e.into()),
    }
}

pub async fn create_user_handler(
    State(app_state): State<Arc<AppState>>,
    Json(body): Json<CreateUserSchema>,
) -> Result<impl IntoResponse, (StatusCode, Json<serde_json::Value>)> {
    //加密
    let salt = SaltString::generate(&mut OsRng);
    let hashed_password = Argon2::default()
    .hash_password(body.password.as_bytes(), &salt)
    .map_err(|e| {
        let error_response = serde_json::json!({
            "status": "fail",
            "message": format!("Error while hashing password: {}", e),
        });
        (StatusCode::INTERNAL_SERVER_ERROR, Json(error_response))
    })
    .map(|hash| hash.to_string())?;
    

    //&body.password = hashed_password;
    match app_state
        .db
        .create_user(&body, hashed_password)
        .await.map_err(MyError::from) 
    {
        Ok(res) => Ok((StatusCode::CREATED, Json(res))),
        Err(e) => Err(e.into()),
    }
}

pub async fn get_user_by_username_handler(
    Path(username): Path<String>,
    headers: HeaderMap,
    State(app_state): State<Arc<AppState>>,
) -> Result<impl IntoResponse, (StatusCode, Json<serde_json::Value>)> {
    let token = headers.get("token");
    if(token.is_none()){
        let error_response = serde_json::json!({
            "status": "fail",
            "message": "Token is empty"
        });
        return Err((StatusCode::UNAUTHORIZED, Json(error_response)));
    }

    let tokenstr = token.unwrap().to_str().unwrap();
    match token::verify_jwt_token(app_state.env.access_token_public_key.to_owned(), &tokenstr)
    {
        Ok(token_details) => token_details,
        Err(e) => {
            let error_response = serde_json::json!({
                "status": "fail",
                "message": format_args!("{:?}", e)
            });
            return Err((StatusCode::UNAUTHORIZED, Json(error_response)));
        }
    };

    match app_state.db.get_user(&username).await.map_err(MyError::from) {
        Ok(res) => Ok(Json(res)),
        Err(e) => Err(e.into()),
    }
}

pub async fn login_user_handler(
    State(app_state): State<Arc<AppState>>,
    Json(body): Json<VaildUserSchema>,
) -> Result<impl IntoResponse, (StatusCode, Json<serde_json::Value>)> {
    log::error!("login_user_handler: {:?}", &body);

    match app_state.db.get_user(&body.username).await.map_err(MyError::from) {
        Ok(res) => {
            //生成token
            let password_key = res.data.user.password.clone();
            let user_id = res.data.user.id.clone();

            let is_valid = match PasswordHash::new(&password_key) {
                Ok(parsed_hash) => Argon2::default()
                    .verify_password(body.password.as_bytes(), &parsed_hash)
                    .map_or(false, |_| true),
                Err(_) => false,
            };

            if !is_valid {
                let error_response = serde_json::json!({
                    "status": "fail",
                    "message": "Invalid password"
                });
                return Err((StatusCode::BAD_REQUEST, Json(error_response)));
            }
            
            let access_token_details = generate_token(
                user_id,
                app_state.env.access_token_max_age,
                app_state.env.access_token_private_key.to_owned(),
            )?;

            let mut response = Response::new(
                json!({"status": "success", "access_token": access_token_details.token.unwrap()})
                    .to_string(),
            );

            let mut headers = HeaderMap::new();
            headers.append(
                header::SET_COOKIE,
                header::HeaderValue::from_static("access_token_details"),
            );
            
            headers.append(
                header::CONTENT_TYPE,
                    header::HeaderValue::from_static("application/json")
            );
            response.headers_mut().extend(headers);
        
            Ok(response)
        }
        Err(e) => {
            log::error!("valid_user_handler: {:?}", e);
            Err(e.into())
        }
    }
}

pub async fn update_user_handler(
    Path(username): Path<String>,
    headers: HeaderMap,
    State(app_state): State<Arc<AppState>>,
    Json(body): Json<UpdateUserSchema>,
) -> Result<impl IntoResponse, (StatusCode, Json<serde_json::Value>)> {
    let token = headers.get("token");
    if(token.is_none()){
        let error_response = serde_json::json!({
            "status": "fail",
            "message": "Token is empty"
        });
        return Err((StatusCode::UNAUTHORIZED, Json(error_response)));
    }

    let tokenstr = token.unwrap().to_str().unwrap();
    match token::verify_jwt_token(app_state.env.access_token_public_key.to_owned(), &tokenstr)
    {
        Ok(token_details) => token_details,
        Err(e) => {
            let error_response = serde_json::json!({
                "status": "fail",
                "message": format_args!("{:?}", e)
            });
            return Err((StatusCode::UNAUTHORIZED, Json(error_response)));
        }
    };

    match app_state
        .db
        .update_user(&username, &body)
        .await
        .map_err(MyError::from)
    {
        Ok(res) => Ok(Json(res)),
        Err(e) => Err(e.into()),
    }
}

pub async fn delete_user_handler(
    Path(username): Path<String>,
    headers: HeaderMap,
    State(app_state): State<Arc<AppState>>
) -> Result<impl IntoResponse, (StatusCode, Json<serde_json::Value>)> {
    //验证token
    let token = headers.get("token");
    if(token.is_none()){
        let error_response = serde_json::json!({
            "status": "fail",
            "message": "Token is empty"
        });
        return Err((StatusCode::UNAUTHORIZED, Json(error_response)));
    }

    let tokenstr = token.unwrap().to_str().unwrap();
    match token::verify_jwt_token(app_state.env.access_token_public_key.to_owned(), &tokenstr)
    {
        Ok(token_details) => token_details,
        Err(e) => {
            let error_response = serde_json::json!({
                "status": "fail",
                "message": format_args!("{:?}", e)
            });
            return Err((StatusCode::UNAUTHORIZED, Json(error_response)));
        }
    };

    match app_state.db.delete_user(&username).await.map_err(MyError::from) {
        Ok(res) => 
        {
            if(res.data.user.username == username
                && res.data.user.is_delete == Some(true))
            {
                let message = MessageResponse {
                    code: 200,
                    message: "success".to_string(),
                };
                return Ok((StatusCode::ACCEPTED, Json(message)))
            }else{
                let message = MessageResponse {
                    code: 200,
                    message: "failure".to_string(),
                };
                return Ok((StatusCode::BAD_REQUEST, Json(message)))
            }

        }
        Err(e) => Err(e.into()),
    }
}

pub async fn logout_user_handler() 
    -> Result<impl IntoResponse, (StatusCode, Json<serde_json::Value>)> {
    let cookie = Cookie::build("token", "")
        .path("/")
        .max_age(time::Duration::hours(-1))
        .same_site(SameSite::Lax)
        .http_only(true)
        .finish();

    let mut headers = HeaderMap::new();
    headers.append(
        "token",
        header::HeaderValue::from_static(""),
    );
    
    let mut response = Response::new(json!({"status": "success"}).to_string());
    response
    .headers_mut()
    .insert(header::SET_COOKIE, cookie.to_string().parse().unwrap());
    response.headers_mut().extend(headers);
    Ok(response)
}