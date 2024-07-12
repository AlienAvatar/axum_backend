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
use crate::crawel::handler::{public_handler, public_crawl_handler, shared_handler};
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};

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

    // 日志追踪
    tracing_subscriber::registry()
        .with(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| "example_multipart_form=debug,tower_http=debug".into()),
        )
        .with(tracing_subscriber::fmt::layer())
        .init();

    let origins = [
        "http://localhost:5173".parse().unwrap(),
        "http://localhost:8080".parse().unwrap(),
        "http://localhost:10002".parse().unwrap(),
        "http://localhost:10003".parse().unwrap(),
        "http://localhost:10001".parse().unwrap(),
    ];
    
    let cors = CorsLayer::new()
        .allow_origin(origins)
        .allow_methods([Method::GET, Method::POST, Method::PATCH, Method::DELETE])
        .allow_credentials(true)
        .allow_headers([AUTHORIZATION, ACCEPT, CONTENT_TYPE, CONTENT_LANGUAGE, ACCEPT_LANGUAGE, token_header]);

    let app = create_router(Arc::new(AppState { db: db.clone(), env: config.clone(), redis_client: redis_client.clone()})).layer(cors);

    println!("🚀 Server started successfully");
    let listener = tokio::net::TcpListener::bind("0.0.0.0:10001").await.unwrap();

    tracing::debug!("listening on {}", listener.local_addr().unwrap());
    axum::serve(listener, app).await.unwrap();

    Ok(())
}

async fn handler_404() -> impl IntoResponse {
    (StatusCode::NOT_FOUND, "nothing to see here")
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
//     // 羌佛文告 95
//     let office_announcement_news_url = "http://www.gufowang.org/office-announcement/";
//     let office_announcement_news_response = reqwest::get(office_announcement_news_url).await.unwrap();
//     let office_announcement_news_html = office_announcement_news_response.text().await.unwrap();
//     let office_announcement_news_document = Html::parse_document(&office_announcement_news_html);
//     public_crawl_handler(office_announcement_news_document, appstate.clone(), "羌佛文告").await;

//     // 总部文告 98
//     let announcement_news_url = "http://www.gufowang.org/temple/announcement/";
//     let announcement_news_response = reqwest::get(announcement_news_url).await.unwrap();
//     let announcement_news_html = announcement_news_response.text().await.unwrap();
//     let announcement_news_document = Html::parse_document(&announcement_news_html);
//     public_crawl_handler(announcement_news_document, appstate.clone(), "总部文告").await;

//     // 大德文集 44
//     let discourse_url = "http://www.gufowang.org/discourse/";
//     let discourse_response = reqwest::get(discourse_url).await.unwrap();
//     let discourse_html = discourse_response.text().await.unwrap();
//     let discourse_document = Html::parse_document(&discourse_html);
//     public_crawl_handler(discourse_document, appstate.clone(), "大德文集").await;

//     // 圣德回复 19
//     let answer_url = "http://www.gufowang.org/temple/answer/";
//     let answer_response = reqwest::get(answer_url).await.unwrap();
//     let answer_html = answer_response.text().await.unwrap();
//     let answer_document = Html::parse_document(&answer_html);
//     public_crawl_handler(answer_document, appstate.clone(), "圣德回复").await;

//      //受用分享 27
//     let benefit_url = "http://www.gufowang.org/benefit/";
//     let benefit_response = reqwest::get(benefit_url).await.unwrap();
//     let benefit_html = benefit_response.text().await.unwrap();
//     let benefit_document = Html::parse_document(&benefit_html);
//     // 访问公告中的li元素
//     public_crawl_handler(benefit_document, appstate.clone(), "受用分享").await;

//     //佛书法著
//     let foshu_url = "http://www.gufowang.org/foshu/";
//     let foshu_response = reqwest::get(foshu_url).await.unwrap();
//     let foshu_html = foshu_response.text().await.unwrap();
//     let foshu_document = Html::parse_document(&foshu_html);
//     // 访问公告中的li元素
//     public_crawl_handler(foshu_document, appstate.clone(), "佛书法著").await;

//     // 正法新闻 56
//     let true_dharma_news_url = "http://www.gufowang.org/true-dharma-news/";
//     let true_dharma_news_response = reqwest::get(true_dharma_news_url).await.unwrap();
//     let true_dharma_news_html = true_dharma_news_response.text().await.unwrap();
//     let true_dharma_news_document = Html::parse_document(&true_dharma_news_html);
//     public_crawl_handler(true_dharma_news_document, appstate.clone(), "正法新闻").await;

//     // 摧邪显正 36
//     let positive_url = "http://www.gufowang.org/positive/";
//     let positive_response = reqwest::get(positive_url).await.unwrap();
//     let positive_html = positive_response.text().await.unwrap();
//     let positive_document = Html::parse_document(&positive_html);
//     //positive_handler(positive_document, appstate.clone()).await;
//     public_crawl_handler(positive_document, appstate.clone(), "摧邪显正").await;


//     //羌佛公告 10
//     let main_url = "http://www.gufowang.org/";
//     let response = reqwest::get(main_url).await.unwrap();
//     let html = response.text().await.unwrap();
//     let document = Html::parse_document(&html);
//     // 访问公告中的li元素
//     public_handler(document, appstate.clone()).await;

//     //古佛降世 10
//         let buddha_url = "http://www.gufowang.org/buddha/";
//         let buddha_response = reqwest::get(buddha_url).await.unwrap();
//         let buddha_html = buddha_response.text().await.unwrap();
//         let buddha_document = Html::parse_document(&buddha_html);
//         //buddha_handler(buddha_document, appstate.clone()).await;
//         public_crawl_handler(buddha_document, appstate.clone(), "古佛降世").await;

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

        // while let Ok(chun_data) =  field.chunk().await{
        //     if let Some(bytes_data) = chun_data {
        //         o_file.write_all(&bytes_data).await.unwrap();
        //     } else {
        //         break;
        //     }
        // }
        
//     }
//     Html("upload successful")
// }
