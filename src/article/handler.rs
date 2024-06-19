use std::{collections::HashMap, sync::Arc};
use axum::{
    extract::{Path, Query, State},
    http::{HeaderMap, StatusCode},
    response::{IntoResponse},
    Json,
};
use mongodb::bson::oid::ObjectId;
use serde_json::{json, Map, Value};
use crate::{
    error::MyError, token::{self, verify_jwt_token, TokenDetails}, 
    user::{model::TokenClaims, response::{MessageResponse, TokenMessageResponse}}, 
    article::schema::{FilterOptions, CreateArticleSchema, UpdateArticleSchema}, 
    AppState
};

pub async fn article_list_handler(
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
    let id = opts.id.unwrap_or("".to_string());
    let title = opts.title.unwrap_or("".to_string());
    let author = opts.author.unwrap_or("".to_string());
    let is_delete = opts.is_delete.unwrap_or(false);
    println!("id: {}", id);")";
    println!("title: {}", title);")";
    println!("author: {}", author);")";
    println!("is_delete: {}", is_delete);")";
    match app_state
        .db
        .fetch_articles(limit, page,title.as_str(), id.as_str(), author.as_str(), &is_delete)
        .await
        .map_err(MyError::from)
    {
        Ok(res) => Ok(Json(res)),
        Err(e) => Err(e.into()),
    }
}

pub async fn create_article_handler(
    State(app_state): State<Arc<AppState>>,
    headers: HeaderMap,
    Json(body): Json<CreateArticleSchema>,
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
        .create_article(&body)
        .await.map_err(MyError::from) 
    {
        Ok(res) => Ok((StatusCode::CREATED, Json(res))),
        Err(e) => Err(e.into()),
    }
}

pub async fn get_article_by_id_handler(
    Path(id): Path<String>,
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

    match app_state.db.get_article(&id).await.map_err(MyError::from) {
        Ok(res) => Ok(Json(res)),
        Err(e) => Err(e.into()),
    }
}

pub async fn update_article_by_id_handler(
    Path(id): Path<String>,
    headers: HeaderMap,
    State(app_state): State<Arc<AppState>>,
    Json(body): Json<UpdateArticleSchema>,
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
        .update_article(&id, &body)
        .await
        .map_err(MyError::from)
    {
        Ok(res) => Ok(Json(res)),
        Err(e) => Err(e.into()),
    }
}

pub async fn delete_article_by_id_handler(
    Path(id): Path<String>,
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

    match app_state.db.delete_article(&id).await.map_err(MyError::from) {
        Ok(res) => 
        {
            if(res.data.article.id == id
                && res.data.article.is_delete == Some(true))
            {
                let message = MessageResponse {
                    code: 200,
                    status: "success".to_string(),
                    message: "delete success".to_string(),
                };
                return Ok((StatusCode::ACCEPTED, Json(message)))
            }else{
                let message = MessageResponse {
                    code: 200,
                    status: "failure".to_string(),
                    message: "delete failure".to_string(),
                };
                return Ok((StatusCode::BAD_REQUEST, Json(message)))
            }
        }
        Err(e) => Err(e.into()),
    }
}

pub async fn delete_article_by_ids_handler(
    Query(map): Query<HashMap<String, String>>,
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
    
    let mut result_array: Vec<serde_json::Value> = Vec::new();
    for (key, value) in map {
        let id = value;
        match app_state.db.delete_article(&id).await.map_err(MyError::from) {
            Ok(res) => 
            {
                if(res.data.article.id == id
                    && res.data.article.is_delete == Some(true))
                {           
                    continue;
                }else{
                    let id_mes = format!("id {:?}", res.data.article.id);
                    let message = id_mes.to_string() + " deleted failure";
                    let message_obj = json!({
                        "code": 200,
                        "status": "failure",
                        "message": message
                    });
                    result_array.push(message_obj);
                }
            }
            Err(e) => {
                let error_obj = json!({
                    "code": 500,
                    "status": "error",
                    "message": e.to_string()
                });
                result_array.push(error_obj);
            }
        }
    };

    let response = json!({
        "code": 200,
        "status": "success",
        "meesage": result_array
    });
    return Ok((StatusCode::ACCEPTED, Json(response)))
}