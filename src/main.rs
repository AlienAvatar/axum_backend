use axum::{extract::Multipart, http::{header::{ACCEPT_LANGUAGE, CONTENT_LANGUAGE}, HeaderName, StatusCode}, response::{ IntoResponse}, routing::{get, post}, Json, Router};
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
use reqwest;
use scraper::{Html, Selector};
use std::rc::Rc;
use crate::crawel::handler::{public_handler, buddha_handler, buddha_dharma_handler, recognition_handler, holy_realization_handler, holy_occurrences_handler, public_crawl_handler};

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
mod crawel;


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


// #[tokio::main]
// async fn main() {
//     // 获取 http://www.gufowang.org/ 网页的 HTML 内容
//     dotenv().ok();

//     let config = Config::init();
    
//     let db = DB::init().await.unwrap();
//     let redis_client = match Client::open(config.redis_url.to_owned()) {
//         Ok(client) => {
//             println!("✅Connection to the redis is successful!");
//             client
//         }
//         Err(e) => {
//             println!("🔥 Error connecting to Redis: {}", e);
//             std::process::exit(1);
//         }
//     };
//     let appstate:Arc<AppState> = Arc::new(AppState { db: db.clone(), env: config.clone(), redis_client: redis_client.clone()});
//     // 解析 HTML 文档
//     //羌佛公告 10
//     let main_url = "http://www.gufowang.org/";
//     let response = reqwest::get(main_url).await.unwrap();
//     let html = response.text().await.unwrap();
//     let document = Html::parse_document(&html);
//     // 访问公告中的li元素
//     public_handler(document, appstate.clone()).await;

//     //古佛降世 10
//     let buddha_url = "http://www.gufowang.org/buddha/";
//     let buddha_response = reqwest::get(buddha_url).await.unwrap();
//     let buddha_html = buddha_response.text().await.unwrap();
//     let buddha_document = Html::parse_document(&buddha_html);
//     //buddha_handler(buddha_document, appstate.clone()).await;
//     public_crawl_handler(buddha_document, appstate.clone(), "古佛降世").await;

//     //羌佛说法 23
//     let buddha_dharma_url = "http://www.gufowang.org/buddha-dharma/";
//     let buddha_dharma_response = reqwest::get(buddha_dharma_url).await.unwrap();
//     let buddha_dharma_html = buddha_dharma_response.text().await.unwrap();
//     let buddha_dharma_document = Html::parse_document(&buddha_dharma_html);
//     public_crawl_handler(buddha_dharma_document, appstate.clone(), "羌佛说法").await;

//     //认证恭祝
//     let recognition_url = "http://www.gufowang.org/recognition/";
//     let recognition_response = reqwest::get(recognition_url).await.unwrap();
//     let recognition_html = recognition_response.text().await.unwrap();
//     let recognition_document = Html::parse_document(&recognition_html);
//     public_crawl_handler(recognition_document, appstate.clone(), "认证恭祝").await;

//     //羌佛圣量
//     let holy_realization_url = "http://www.gufowang.org/holy-realization/";
//     let holy_realization_response = reqwest::get(holy_realization_url).await.unwrap();
//     let holy_realization_html = holy_realization_response.text().await.unwrap();
//     let holy_realization_document = Html::parse_document(&holy_realization_html);
//     public_crawl_handler(holy_realization_document, appstate.clone(), "羌佛圣量").await;

//     //羌佛圣迹
//     let holy_occurrences_url = "http://www.gufowang.org/holy-occurrences/";
//     let holy_occurrences_response = reqwest::get(holy_occurrences_url).await.unwrap();
//     let holy_occurrences_html = holy_occurrences_response.text().await.unwrap();
//     let holy_occurrences_document = Html::parse_document(&holy_occurrences_html);
//     public_crawl_handler(holy_occurrences_document, appstate.clone(), "羌佛圣迹").await;
    
//     //圆满佛格 27
//     let buddha_virtue_url = "http://www.gufowang.org/buddha-virtue/";
//     let buddha_virtue_response = reqwest::get(buddha_virtue_url).await.unwrap();
//     let buddha_virtue_html = buddha_virtue_response.text().await.unwrap();
//     let buddha_virtue_document = Html::parse_document(&buddha_virtue_html);
//     public_crawl_handler(buddha_virtue_document, appstate.clone(), "圆满佛格").await;

//     //妙谙五明 34
//     let wuming_url = "http://www.gufowang.org/wuming/";
//     let wuming_response = reqwest::get(wuming_url).await.unwrap();
//     let wuming_html = wuming_response.text().await.unwrap();
//     let wuming_document = Html::parse_document(&wuming_html);
//     public_crawl_handler(wuming_document, appstate.clone(), "妙谙五明").await;

//     //渡生成就 34
//     let savelivingbings_url = "http://www.gufowang.org/savelivingbings/";
//     let savelivingbings_response = reqwest::get(savelivingbings_url).await.unwrap();
//     let savelivingbings_html = savelivingbings_response.text().await.unwrap();
//     let savelivingbings_document = Html::parse_document(&savelivingbings_html);
//     public_crawl_handler(savelivingbings_document, appstate.clone(), "渡生成就").await;

// }


// async fn upload_page() -> impl IntoResponse {
//     Html(
//         r#"
//         <body>
//             <form action="do_upload" method="post" enctype="multipart/form-data">
//                 <input type="file" name="uploadFile">
//                 <input type="submit" value="Upload">
//             </form>
//         <body>
//         "#        
//     )
// }

// async fn do_upload(mut multipart: Multipart) -> impl IntoResponse {

//     while let Some(mut field) = multipart.next_field().await.expect("next field failed") {
//         let filename = field.file_name().unwrap().to_string();

//         let mut o_file = fs::File::create(&filename).await.unwrap();

//         while let Ok(chun_data) =  field.chunk().await{
//             if let Some(bytes_data) = chun_data {
//                 o_file.write_all(&bytes_data).await.unwrap();
//             } else {
//                 break;
//             }
//         }
        
//     }
//     Html("upload successful")
// }



// fn init_logger() {
//     use chrono::Local;
//     use std::io::Write;

//     let env = env_logger::Env::default()
//         .filter_or(env_logger::DEFAULT_FILTER_ENV, "info");
//     // 设置日志打印格式
//     env_logger::Builder::from_env(env)
//         .format(|buf, record| {
//             writeln!(
//                 buf,
//                 "{} {} [{}] {}",
//                 Local::now().format("%Y-%m-%d %H:%M:%S"),
//                 record.level(),
//                 record.module_path().unwrap_or("<unnamed>"),
//                 &record.args()
//             )
//         })
//         .init();
//     log::info!("env_logger initialized.");
// }