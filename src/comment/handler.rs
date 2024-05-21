use std::sync::Arc;
use axum::{
    extract::{Path, Query, State},
    http::{header, HeaderMap, HeaderValue, StatusCode},
    response::{IntoResponse, Response},
    Json,
};
use mongodb::bson::oid::ObjectId;

use crate::{
    error::MyError, token::{self, verify_jwt_token, TokenDetails}, comment::{
         schema::{CreateCommentSchema, FilterOptions, UpdateCommentSchema}
    }, AppState
};
use jsonwebtoken::{decode, encode, Algorithm, EncodingKey, Header};
use chrono::{DateTime, Utc};
use axum_extra::extract::cookie::{Cookie, SameSite, CookieJar};
use argon2::{password_hash::SaltString, Argon2, PasswordHash, PasswordHasher, PasswordVerifier};
use rand_core::OsRng;
use serde_json::json;
use redis::AsyncCommands;

pub async fn create_comment_handler(
    State(app_state): State<Arc<AppState>>,
    headers: HeaderMap,
    Json(body): Json<CreateCommentSchema>,
) -> Result<impl IntoResponse, (StatusCode, Json<serde_json::Value>)> {
     //valid token
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
         .create_comment(&body)
         .await.map_err(MyError::from) 
     {
         Ok(res) => Ok((StatusCode::CREATED, Json(res))),
         Err(e) => Err(e.into()),
     }
}

pub async fn comment_list_handler(
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
        .fetch_comments(limit, page)
        .await
        .map_err(MyError::from)
    {
        Ok(res) => Ok(Json(res)),
        Err(e) => Err(e.into()),
    }
}