use std::{collections::HashMap, sync::Arc};
use tokio::{self, fs, io::AsyncWriteExt};
use axum::{
    extract::{Path, Query, State, Multipart, Request, multipart},
    http::StatusCode,
    http::header::{HeaderMap, HeaderValue, HeaderName},
    body::Bytes,
    response::{Html, Redirect},
    response::IntoResponse,
    BoxError, Json,
};
use mongodb::bson::Array;
use tokio_util::io::StreamReader;
use crate::{
    error::MyError,
    note::schema::{CreateNoteSchema, FilterOptions, UpdateNoteSchema},
    AppState,
};
use serde::{Deserialize, Serialize};
use futures::{Stream, TryStreamExt};
use tokio::{fs::File, io::BufWriter};
use std::io;
use rand::prelude::random;
use std::fs::read;
use std::net::{IpAddr, Ipv4Addr, Ipv6Addr};
use local_ip_address::local_ip;

const UPLOADS_DIRECTORY: &str = "img";
#[derive(Deserialize)]
pub struct SubjectArgsOpt {
    pub page: Option<i32>,
    pub keyword: Option<String>,
}

pub async fn health_checker_handler(
    Query(args): Query<HashMap<String, String>>
) -> impl IntoResponse {
    let a = format!("{:?}", args);
    println!("{:?}", args);
    // let page = args.page.unwrap_or(0);
    // let keyword = args.keyword.unwrap_or("".to_string());
    // format!("Page {}, keyword: {} of subjects", page, keyword);
    // println!("page {}",page);
    // println!("keyword {}",keyword);

    const MESSAGE: &str = "RESTful API in Rust using Axum Framework and MongoDB";

    let json_response = serde_json::json!({
        "status": "success",
        "message": MESSAGE
    });

    Json(json_response)
}


pub async fn show_form() -> Html<&'static str> {
    Html(
        r#"
        <!doctype html>
        <html>
            <head></head>
            <body>
                <form action="/api/note/test/" method="post" enctype="multipart/form-data">
                    <label>
                        Upload file:
                        <input id="input_file" type="file" name="file" multiple>
                    </label>

                    <input type="submit" value="Upload files">
                </form>
            </body>
        </html>
        "#,
    )
}

pub async fn accept_form(
    mut multipart: Multipart
) -> Result<impl IntoResponse, (StatusCode, Json<serde_json::Value>)> {
    while let Some(mut field) = multipart.next_field().await.unwrap() {
        // let name = field.name().unwrap().to_string();
        // let mut o_file = fs::File::create(&name).await.unwrap();

        let filename: String = field.file_name().unwrap().to_string();
        //文件类型
        let content_type = field.content_type().unwrap().to_string();

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
            let my_local_ip = local_ip();  
            
            let url: String = format!("http:://{}:10001/show_image/{}.{}", my_local_ip.unwrap(), rnd, ext_name);
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

async fn redirect(path: String) -> Result<(StatusCode, HeaderMap), String> {
    let mut headers = HeaderMap::new();
    //重设LOCATION，跳到新页面
    headers.insert(
        axum::http::header::LOCATION,
        HeaderValue::from_str(&path).unwrap(),
    );
    //302重定向
    Ok((StatusCode::FOUND, headers))
}
// pub async fn serve_image(Path(filename): Path<String>) -> Response<BoxBody> {
//     let img_folder = std::path::Path::new("img");
//     let file_path = img_folder.join(&filename);

//     if let Ok(file) = fs::File::open(&file_path).await {
//         Response::builder()
//             .header("Content-Type", "image/jpeg") // or appropriate content type
//             .body(BoxBody::from(file))
//             .unwrap()
//     } else {
//         Response::builder()
//             .status(StatusCode::NOT_FOUND)
//             .body(BoxBody::default())
//             .unwrap()
//     }
// }

pub async fn show_img(
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

pub async fn note_list_handler(
    opts: Option<Query<FilterOptions>>,
    State(app_state): State<Arc<AppState>>,
) -> Result<impl IntoResponse, (StatusCode, Json<serde_json::Value>)> {
    let Query(opts) = opts.unwrap_or_default();

    let limit = opts.limit.unwrap_or(10) as i64;
    let page = opts.page.unwrap_or(1) as i64;

    match app_state
        .db
        .fetch_notes(limit, page)
        .await
        .map_err(MyError::from)
    {
        Ok(res) => Ok(Json(res)),
        Err(e) => Err(e.into()),
    }
}

pub async fn create_note_handler(
    State(app_state): State<Arc<AppState>>,
    Json(body): Json<CreateNoteSchema>,
) -> Result<impl IntoResponse, (StatusCode, Json<serde_json::Value>)> {
    match app_state.db.create_note(&body).await.map_err(MyError::from) {
        Ok(res) => Ok((StatusCode::CREATED, Json(res))),
        Err(e) => Err(e.into()),
    }
}

pub async fn get_note_handler(
    Path(id): Path<String>,
    State(app_state): State<Arc<AppState>>,
) -> Result<impl IntoResponse, (StatusCode, Json<serde_json::Value>)> {
    match app_state.db.get_note(&id).await.map_err(MyError::from) {
        Ok(res) => Ok(Json(res)),
        Err(e) => Err(e.into()),
    }
}

pub async fn edit_note_handler(
    Path(id): Path<String>,
    State(app_state): State<Arc<AppState>>,
    Json(body): Json<UpdateNoteSchema>,
) -> Result<impl IntoResponse, (StatusCode, Json<serde_json::Value>)> {
    match app_state
        .db
        .edit_note(&id, &body)
        .await
        .map_err(MyError::from)
    {
        Ok(res) => Ok(Json(res)),
        Err(e) => Err(e.into()),
    }
}

pub async fn delete_note_handler(
    Path(id): Path<String>,
    State(app_state): State<Arc<AppState>>,
) -> Result<impl IntoResponse, (StatusCode, Json<serde_json::Value>)> {
    match app_state.db.delete_note(&id).await.map_err(MyError::from) {
        Ok(_) => Ok(StatusCode::NO_CONTENT),
        Err(e) => Err(e.into()),
    }
}
