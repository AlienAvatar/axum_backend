use std::{collections::HashMap, sync::Arc};
use axum::{
    extract::{Path, Query, State, Multipart},
    http::{HeaderMap, HeaderValue, StatusCode, HeaderName},
    response::IntoResponse,
    Json,
};
use serde_json::{json, Value};
use crate::{
    error::MyError, token::{self}, 
    user::response::MessageResponse, 
    article::schema::{FilterOptions, CreateArticleSchema, UpdateArticleSchema}, 
    article::model::UpdateArticleModel,
    AppState
};
use scraper::{Html, Selector};
use rand::prelude::random;
use super::{response::CompackResponse, schema::UpdateArticleSupportUserSchema};
use std::fs::read;

pub async fn article_list_handler(
    opts: Option<Query<FilterOptions>>,
    headers: HeaderMap,
    State(app_state): State<Arc<AppState>>,
) -> Result<impl IntoResponse, (StatusCode, Json<serde_json::Value>)> {
    let Query(opts) = opts.unwrap_or_default();

    let limit = opts.limit.unwrap_or(10) as i64;
    let page = opts.page.unwrap_or(1) as i64;
    let id = opts.id.unwrap_or("".to_string());
    let title = opts.title.unwrap_or("".to_string());
    let author = opts.author.unwrap_or("".to_string());
    let category = opts.category.unwrap_or("".to_string());
    let is_delete = opts.is_delete.unwrap_or(false);

    match app_state
        .db
        .fetch_articles(limit, page,id.as_str(), title.as_str(), author.as_str(), category.as_str(), &is_delete)
        .await
        .map_err(MyError::from)
    {
        
        Ok(res) => {
            return Ok(Json(res))
        },
        Err(e) => Err(e.into()),
    }
}


async fn fetch_articles_by_category(
    app_state: Arc<AppState>,
    limit: i64,
    page: i64,
    category: &str,
    is_delete: bool,
    vec: &mut Vec<Value>,
    // count: &mut usize,
) {
    let articles = app_state
        .db
        .fetch_articles_page(limit, page, "", "", "", category, &is_delete)
        .await
        .map_err(MyError::from);

    let res_list = serde_json::json!({
        "category": category,
        "data": &articles.unwrap(),
        
    });

    vec.push(res_list);
}

pub async fn article_home_list_handler(
    opts: Option<Query<FilterOptions>>,
    State(app_state): State<Arc<AppState>>,
) -> Result<impl IntoResponse, (StatusCode, Json<serde_json::Value>)> {
    let Query(opts) = opts.unwrap_or_default();

    let limit = opts.limit.unwrap_or(10) as i64;
    let page = opts.page.unwrap_or(1) as i64;
    let is_delete = false;
    
    let mut res_vec: Vec<Value> = vec![];
    let mut category = "古佛降世";

    // match app_state
    //     .db
    //     .fetch_articles(limit, page,"", "", "", category, &is_delete)
    //     .await
    //     .map_err(MyError::from)
    // {
    //     Ok(res) =>{
    //         let res_list = serde_json::json!({
    //             "category": category,
    //             "data" : res.articles
    //         });
    //         res_vec.push(res_list);
    //     }
    //     Err(e) => {
    //         return Err(e.into())
    //     },
    // }
    fetch_articles_by_category(
        app_state.clone(),
        limit,
        page,
        category,
        is_delete,
        &mut res_vec
    ).await;
    category = "羌佛说法";
    fetch_articles_by_category(
        app_state.clone(),
        limit,
        page,
        category,
        is_delete,
        &mut res_vec
    ).await;
    category = "公告";
    fetch_articles_by_category(
        app_state.clone(),
        limit,
        page,
        category,
        is_delete,
        &mut res_vec
    ).await;
    category = "认证恭祝";
    fetch_articles_by_category(
        app_state.clone(),
        limit,
        page,
        category,
        is_delete,
        &mut res_vec
    ).await;
    category = "羌佛圣量";
    fetch_articles_by_category(
        app_state.clone(),
        limit,
        page,
        category,
        is_delete,
        &mut res_vec
    ).await;
    category = "羌佛圣迹";
    fetch_articles_by_category(
        app_state.clone(),
        limit,
        page,
        category,
        is_delete,
        &mut res_vec
    ).await;
    category = "圆满佛格";
    fetch_articles_by_category(
        app_state.clone(),
        limit,
        page,
        category,
        is_delete,
        &mut res_vec
    ).await;
    category = "妙谙五明";
    fetch_articles_by_category(
        app_state.clone(),
        limit,
        page,
        category,
        is_delete,
        &mut res_vec
    ).await;
    category = "渡生成就";
    fetch_articles_by_category(
        app_state.clone(),
        limit,
        page,
        category,
        is_delete,
        &mut res_vec
    ).await;
    category = "正法新闻";
    fetch_articles_by_category(
        app_state.clone(),
        limit,
        page,
        category,
        is_delete,
        &mut res_vec
    ).await;
    category = "摧邪显正";
    fetch_articles_by_category(
        app_state.clone(),
        limit,
        page,
        category,
        is_delete,
        &mut res_vec
    ).await;
    category = "受用分享";
    fetch_articles_by_category(
        app_state.clone(),
        limit,
        page,
        category,
        is_delete,
        &mut res_vec
    ).await;
    category = "佛书法著";
    fetch_articles_by_category(
        app_state.clone(),
        limit,
        page,
        category,
        is_delete,
        &mut res_vec
    ).await;
    category = "羌佛文告";
    fetch_articles_by_category(
        app_state.clone(),
        limit - 5,
        page,
        category,
        is_delete,
        &mut res_vec
    ).await;
    category = "总部文告";
    fetch_articles_by_category(
        app_state.clone(),
        limit - 5,
        page,
        category,
        is_delete,
        &mut res_vec
    ).await;
    category = "大德文集";
    fetch_articles_by_category(
        app_state.clone(),
        limit - 5,
        page,
        category,
        is_delete,
        &mut res_vec
    ).await;
    category = "圣德回复";
    fetch_articles_by_category(
        app_state.clone(),
        limit - 5,
        page,
        category,
        is_delete,
        &mut res_vec
    ).await;

    let res_response = serde_json::json!({
        "status": "success",
        "code": 200,
        "data" : res_vec
    });
    Ok(Json(res_response))
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

pub async fn update_cover_img_by_id_handle(
    mut multipart: Multipart,
    headers: HeaderMap,
    State(app_state): State<Arc<AppState>>,
    Path(id): Path<String>,
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

    match app_state.db.get_article(&id).await.map_err(MyError::from){
        Ok(res) => {
            while let Some(mut field) = multipart.next_field().await.unwrap() {
                let filename: String = field.file_name().unwrap().to_string();
                println!("filename: {}", filename);
                //文件类型
                let content_type = field.content_type().unwrap().to_string();
                println!("content_type: {}", content_type);
                
                if content_type.starts_with("image/") {
                    //根据文件类型生成随机文件名(出于安全考虑)
                    let rnd = (random::<f32>() * 1000000000 as f32) as i32;
                     //提取"/"的index位置
                     let index = content_type
                        .find("/")
                        .map(|i| i)
                        .unwrap_or(usize::max_value());
        
                    //文件扩展名
                    let mut ext_name = "xxx";
                    if index != usize::max_value() {
                        ext_name = &content_type[index + 1..];
                    }
        
                    //文件存储路径
                    let save_filename = format!("{}/{}.{}", "img", rnd, ext_name);
                    //文件内容
                    let data = field.bytes().await.unwrap();
        
                    println!("filename:{},content_type:{}", save_filename, content_type);
        
                    let _write_img = tokio::fs::write(&save_filename, &data)
                    .await
                    .map_err(|err| err.to_string());
        
                    //获取本地ip地址
                    let url: String = format!("http://localhost:10001/show_image/{}.{}", rnd, ext_name);
                    let update_body = UpdateArticleModel{
                        title : res.data.article.title,
                        author : res.data.article.author,
                        content : res.data.article.content,
                        support_count : res.data.article.support_count,
                        support_users : res.data.article.support_users,
                        category : res.data.article.category,
                        views_count : res.data.article.views_count,
                        cover_img: url,
                    };

                    match app_state
                        .db
                        .update_article(&id, &update_body)
                        .await
                        .map_err(MyError::from)
                    {
                        Ok(res) => {
                            let response = serde_json::json!({
                                "status": "success",
                                "message": "url",
                                "data": res,
                            });
                            return Ok((StatusCode::ACCEPTED, Json(response)));
                        }
                        Err(e) => {
                            let response = serde_json::json!({
                                "status": "error",
                                "message": "update error",
                            });
                            return Ok((StatusCode::ACCEPTED, Json(response)));
                        }
                    }
                 
                }
            }
        }
        Err(e) => {
            let response = serde_json::json!({
                "status": "error",
                "message": "not found data",
            });
            return Ok((StatusCode::ACCEPTED, Json(response)));;
        }
    }
    
    let response = serde_json::json!({
        "status": "error",
        "message": "No file was uploaded"
    });
    Ok((StatusCode::ACCEPTED, Json(response)))
}

pub async fn upload_img_handle(
    mut multipart: Multipart
) -> Result<impl IntoResponse, (StatusCode, Json<serde_json::Value>)> {
    while let Some(mut field) = multipart.next_field().await.unwrap() {
        // let name = field.name().unwrap().to_string();
        // let mut o_file = fs::File::create(&name).await.unwrap();

        let filename: String = field.file_name().unwrap().to_string();
        println!("filename: {}", filename);
        //文件类型
        let content_type = field.content_type().unwrap().to_string();
        println!("content_type: {}", content_type);
        
        if content_type.starts_with("image/") {
            //根据文件类型生成随机文件名(出于安全考虑)
            let rnd = (random::<f32>() * 1000000000 as f32) as i32;
             //提取"/"的index位置
             let index = content_type
                .find("/")
                .map(|i| i)
                .unwrap_or(usize::max_value());

            //文件扩展名
            let mut ext_name = "xxx";
            if index != usize::max_value() {
                ext_name = &content_type[index + 1..];
            }

            //文件存储路径
            let save_filename = format!("{}/{}.{}", "img", rnd, ext_name);
            //文件内容
            let data = field.bytes().await.unwrap();

            println!("filename:{},content_type:{}", save_filename, content_type);

            let _write_img = tokio::fs::write(&save_filename, &data)
            .await
            .map_err(|err| err.to_string());

            //获取本地ip地址
            let url: String = format!("http://localhost:10001/show_image/{}.{}", rnd, ext_name);
            let response = serde_json::json!({
                "status": "success",
                "message": url,
            });
            return Ok((StatusCode::ACCEPTED, Json(response)));
        }
    }
    let response = serde_json::json!({
        "status": "error",
        "message": "No file was uploaded"
    });
    Ok((StatusCode::ACCEPTED, Json(response)))
}

pub async fn show_img_handle(
    Path(filename): Path<String>,
) -> (HeaderMap, Vec<u8>) {
    let index = filename.find(".").map(|i| i).unwrap_or(usize::max_value());
    //文件扩展名
    let mut ext_name = "xxx";
    if index != usize::max_value() {
        ext_name = &filename[index + 1..];
    }
    let content_type = format!("image/{}", ext_name);
    let mut headers = HeaderMap::new();
    headers.insert(
        HeaderName::from_static("content-type"),
        HeaderValue::from_str(&content_type).unwrap(),
    );
    let file_name = format!("{}/{}", "img", filename);
    (headers, read(&file_name).unwrap())
}

pub async fn get_article_by_id_handler(
    Path(id): Path<String>,
    State(app_state): State<Arc<AppState>>,
) -> Result<impl IntoResponse, (StatusCode, Json<serde_json::Value>)> {
    let comment_data = app_state.db.fetch_comments_by_aritcle_id(&id, 1, 6).await;
    let test = comment_data.unwrap();

    //To do 获取评论
    // let comment_list_data = app_state
    //     .db
    //     .fetch_comments_by_aritcle_id(&id, 1, 6)
    //     .await
    //     .map_err(MyError::from);
    // dbg!(comment_list_data.unwrap());

    match app_state.db.get_article(&id).await.map_err(MyError::from) {
        Ok(article_res) => {
            let compack_data = CompackResponse{
                article: article_res,
                comment_list: test,
            };
            //未异步就直接返回了
            let success_response = serde_json::json!({
                "status": "success",
                "data": compack_data,
                "message": format!("{:?}", "读取成功")
            });
            return Ok(Json(success_response));
        },
        Err(e) => Err(e.into())
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

    // let get_article = app_state.db.get_article(&id).await;

    // let mut support_user_list = get_article.unwrap().data.article.support_users;
    // if(!support_user_list.contains(&body.support_users.to_string())){
    //     support_user_list.push(body.support_users);
    // }
    
    let update_body = UpdateArticleModel{
        title : body.title,
        author : body.author,
        content : body.content,
        support_count : body.support_count,
        support_users : body.support_users,
        category : body.category,
        views_count : body.views_count,
        cover_img: body.cover_img,
    };

    match app_state
        .db
        .update_article(&id, &update_body)
        .await
        .map_err(MyError::from)
    {
        Ok(res) => Ok(Json(res)),
        Err(e) => Err(e.into()),
    }
}

pub async fn update_support_count_by_id_handler(
    Path(id): Path<String>,
    headers: HeaderMap,
    State(app_state): State<Arc<AppState>>,
    Json(body): Json<UpdateArticleSupportUserSchema>,
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

    let get_article = app_state.db.get_article(&id).await;
    let support_article = get_article.unwrap().data.article;

    let mut support_user_list = support_article.support_users;
    let is_exsist = support_user_list.clone().contains(&body.support_user);
    //判断当前user是否已经支持过
    if(is_exsist){
        let response = serde_json::json!({
            "status": "conflict",
            "message": format_args!("{:?}", "This user has already supported this article")
        });
        return Ok(Json(response))
    }

    support_user_list.push(body.support_user);
    let update_body = UpdateArticleModel{
        title : support_article.title,
        author : support_article.author,
        content : support_article.content,
        support_count : support_article.support_count + 1,
        support_users : support_user_list,
        category : support_article.category,
        views_count : support_article.views_count,
        cover_img: support_article.cover_img,
    };

    match app_state
        .db
        .update_article(&id, &update_body)
        .await
        .map_err(MyError::from)
    {
        Ok(res) => Ok(Json(serde_json::to_value(res).unwrap())),
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

