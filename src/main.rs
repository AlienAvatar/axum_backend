use axum::{response::{IntoResponse, Html}, routing::{get, post}, Router, Json, extract::Multipart, http::StatusCode};
use tokio::{self, fs, io::AsyncWriteExt};
use serde::{Deserialize, Serialize};
use std::net::SocketAddr;
use crate::models::user::Person;

use crate::service::login::get_person;
use crate::service::login::create_user;
use crate::mapper::userMapper::insert_user;
//use crate::service::login::add_person;

mod service;
mod models;
mod mapper;
use std::sync::Arc;
use tokio::sync::Mutex;

//mongoDB
extern crate mongodb;
use mongodb::bson::doc;
use mongodb::{Client, options::ClientOptions};

#[tokio::main]
async fn main() {

    let app = Router::new()
        .route("/login/getUser", get(get_person))
        .route("/login/addUser", post(create_user))
        .route("/upload_page", get(upload_page))
        .route("/do_upload", post(do_upload));
    //let app = Router::new().route("/", get(home));

    // run it
    let listener = tokio::net::TcpListener::bind("127.0.0.1:10001")
        .await
        .unwrap();
    println!("listening on {}", listener.local_addr().unwrap());
    axum::serve(listener, app).await.unwrap();
}

async fn insert_document(client: &Client, db_name: &str, coll_name: &str) {
    let db = client.database(db_name);
    let coll = db.collection(coll_name);
    let doc = doc! { "name": "John", "age": 30 };
    coll.insert_one(doc, None).await.unwrap();
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

async fn create_collection(client: &Client, db_name: &str, coll_name: &str) {
    let db = client.database(db_name);
    db.create_collection(coll_name, None).await.unwrap();
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