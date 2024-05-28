use axum::{extract::Multipart, http::{header::{ACCEPT_LANGUAGE, CONTENT_LANGUAGE}, HeaderName, StatusCode}, response::{Html, IntoResponse}, routing::{get, post}, Json, Router};
use tokio::{self, fs, io::AsyncWriteExt};
use config::Config;
use serde::{Deserialize, Serialize};
//mongoDB
use mongodb::{bson::{doc, Document}, Collection};
use mongodb::{options::ClientOptions, options::FindOptions};
// redis
use redis::Client;

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

mod user;
mod error;
mod route;
mod note;
mod db;
mod config;
mod token;
mod article;
mod common;
mod comment;

pub struct AppState {
    db: DB,
    env: Config,
    redis_client: Client,
}

#[tokio::main]
async fn main() -> Result<(), MyError> {
    dotenv().ok();

    let db = DB::init().await?;
    let config = Config::init();

    let redis_client = match Client::open(config.redis_url.to_owned()) {
        Ok(client) => {
            println!("✅Connection to the redis is successful!");
            client
        }
        Err(e) => {
            println!("🔥 Error connecting to Redis: {}", e);
            std::process::exit(1);
        }
    };
    // 创建一个表示"token"头部的HeaderValue实例
    let token_header = "token".parse::<HeaderName>().unwrap();

    let cors = CorsLayer::new()
        .allow_origin("http://localhost:5173".parse::<HeaderValue>().unwrap())
        .allow_origin("http://localhost:8080".parse::<HeaderValue>().unwrap())
        .allow_methods([Method::GET, Method::POST, Method::PATCH, Method::DELETE])
        .allow_credentials(true)
        .allow_headers([AUTHORIZATION, ACCEPT, CONTENT_TYPE, CONTENT_LANGUAGE, ACCEPT_LANGUAGE, token_header]);

    let app = create_router(Arc::new(AppState { db: db.clone(), env: config.clone(), redis_client: redis_client.clone()})).layer(cors);

    println!("🚀 Server started successfully");
    let listener = tokio::net::TcpListener::bind("0.0.0.0:10001").await.unwrap();

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
    log::info!("env_logger initialized.");
}