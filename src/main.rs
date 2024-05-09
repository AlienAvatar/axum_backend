use axum::{response::{IntoResponse, Html}, routing::{get, post}, Router, Json, extract::Multipart, http::StatusCode};
use tokio::{self, fs, io::AsyncWriteExt};

use serde::{Deserialize, Serialize};

//mongoDB
use mongodb::{bson::{doc, Document}, Collection};
use mongodb::{Client, options::ClientOptions, options::FindOptions};
//log
use log::info;

mod user;
mod error;
mod route;
mod note;
mod db;

use std::sync::Arc;
use db::DB;

use axum::http::{
    header::{ACCEPT, AUTHORIZATION, CONTENT_TYPE},
    HeaderValue, Method,
};
use dotenv::dotenv;
use error::MyError;
use route::create_router;
use tower_http::cors::CorsLayer;

pub struct AppState {
    db: DB,
}

#[tokio::main]
async fn main() -> Result<(), MyError> {
    dotenv().ok();

    let db = DB::init().await?;

    let cors = CorsLayer::new()
        .allow_origin("http://localhost:3000".parse::<HeaderValue>().unwrap())
        .allow_methods([Method::GET, Method::POST, Method::PATCH, Method::DELETE])
        .allow_credentials(true)
        .allow_headers([AUTHORIZATION, ACCEPT, CONTENT_TYPE]);

    let app = create_router(Arc::new(AppState { db: db.clone() })).layer(cors);

    println!("🚀 Server started successfully");
    let listener = tokio::net::TcpListener::bind("0.0.0.0:8000").await.unwrap();

    axum::serve(listener, app).await.unwrap();

    Ok(())
}

async fn upload_page() -> impl IntoResponse {
    Html(
        r#"
        <body>
            <form action="do_upload" method="post" enctype="multipart/form-data">
                <input type="file" name="uploadFile">
                <input type="submit" value="Upload">
            </form>
        <body>
        "#        
    )
}

async fn do_upload(mut multipart: Multipart) -> impl IntoResponse {

    while let Some(mut field) = multipart.next_field().await.expect("next field failed") {
        let filename = field.file_name().unwrap().to_string();

        let mut o_file = fs::File::create(&filename).await.unwrap();

        while let Ok(chun_data) =  field.chunk().await{
            if let Some(bytes_data) = chun_data {
                o_file.write_all(&bytes_data).await.unwrap();
            } else {
                break;
            }
        }
        
    }
    Html("upload successful")
}



fn init_logger() {
    use chrono::Local;
    use std::io::Write;

    let env = env_logger::Env::default()
        .filter_or(env_logger::DEFAULT_FILTER_ENV, "info");
    // 设置日志打印格式
    env_logger::Builder::from_env(env)
        .format(|buf, record| {
            writeln!(
                buf,
                "{} {} [{}] {}",
                Local::now().format("%Y-%m-%d %H:%M:%S"),
                record.level(),
                record.module_path().unwrap_or("<unnamed>"),
                &record.args()
            )
        })
        .init();
    info!("env_logger initialized.");
}