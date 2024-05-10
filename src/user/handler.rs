use std::sync::Arc;
use axum::{
    extract::{Path, Query, State},
    http::{StatusCode, HeaderMap},
    response::IntoResponse,
    Json,
};

use crate::{
    error::MyError,
    user::{
        response::{TokenMessageResponse, MessageResponse},
        schema::{CreateUserSchema, FilterOptions, UpdateUserSchema, VaildUserSchema},
        model::{Claims}
    },
    AppState,
};
use serde::{Deserialize, Serialize};
use jsonwebtoken::{decode, encode, Algorithm, EncodingKey, Header};
use chrono::{DateTime, Utc};

pub async fn user_list_handler(
    opts: Option<Query<FilterOptions>>,
    State(app_state): State<Arc<AppState>>,
) -> Result<impl IntoResponse, (StatusCode, Json<serde_json::Value>)> {
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

    // let password_hash = encryption::hash_password(&body.password)
    //         .await
    //         .map_err(internal_error_dyn)?;

    match app_state
        .db
        .create_user(&body)
        .await.map_err(MyError::from) 
    {
        Ok(res) => Ok((StatusCode::CREATED, Json(res))),
        Err(e) => Err(e.into()),
    }
}

pub async fn get_user_by_username_handler(
    Path(username): Path<String>,
    State(app_state): State<Arc<AppState>>,
) -> Result<impl IntoResponse, (StatusCode, Json<serde_json::Value>)> {
    match app_state.db.get_user(&username).await.map_err(MyError::from) {
        Ok(res) => Ok(Json(res)),
        Err(e) => Err(e.into()),
    }
}

pub async fn valid_user_handler(
    State(app_state): State<Arc<AppState>>,
    Json(body): Json<VaildUserSchema>,
) -> Result<impl IntoResponse, (StatusCode, Json<serde_json::Value>)> {
    log::error!("valid_user_handler: {:?}", &body);

    match app_state.db.get_user(&body.username).await.map_err(MyError::from) {
        Ok(res) => {
            if(res.data.user.password == body.password){
            //if(encryption::verify_password(res.data.user.password, body.password)){
                //生成token
                let password_key = res.data.user.password.clone();
                let username_key = res.data.user.username.clone();
                
                let my_claims = Claims {
                    sub: password_key.to_owned(),
                    company: username_key.to_owned(),
                    created_at: Utc::now(),
                };

                let token_str = encode(&Header::default(), &my_claims, &EncodingKey::from_secret("secret".as_ref())).unwrap();
                let message = TokenMessageResponse {
                    code: 200,
                    token: token_str,
                    message: "success".to_string(),
                };
                
                Ok((StatusCode::ACCEPTED, Json(message)))
            }else{
                let message = TokenMessageResponse {
                    code: 200,
                    token: "".to_string(),
                    message: "failure".to_string(),
                };
                Ok((StatusCode::BAD_REQUEST, Json(message)))
            }
        }
        Err(e) => {
            log::error!("valid_user_handler: {:?}", e);
            Err(e.into())
        }
    }
}

pub async fn update_user_handler(
    Path(username): Path<String>,
    State(app_state): State<Arc<AppState>>,
    Json(body): Json<UpdateUserSchema>,
) -> Result<impl IntoResponse, (StatusCode, Json<serde_json::Value>)> {
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
    dbg!(token);
    if(token.is_none()){
        let message = MessageResponse {
            code: 200,
            message: "failure".to_string(),
        };
        return Ok((StatusCode::BAD_REQUEST, Json(message)))
    }

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

pub async fn protected(token: String){
    
}