use axum::{response::Html, routing::get, Router, Json, response::IntoResponse, http::StatusCode};
use crate::models::user::Person;
use crate::models::user::User;
use serde::{Deserialize};

use crate::mapper::userMapper::insert_user;
//mongoDB
extern crate mongodb;
use mongodb::bson::doc;
use mongodb::{Client, options::ClientOptions};

pub async fn get_person() -> Json<Person> {
    let person = Person {
        name: "Alice".to_string(),
        age: 30,
        email: "alice@example.com".to_string(),
        username: "alice".to_string(),
        password: "<PASSWORD>".to_string(),
    };

    Json(person)
}

pub async fn add_person(Json(user): Json<Person>) -> impl IntoResponse {
    // 从请求参数中获取用户名和密码
    let user = Person {
        name: "Alice".to_string(),
        age: 30,
        email: "alice@example.com".to_string(),
        password: "<PASSWORD>".to_string(),
        username: user.username,
    };

    // 执行添加用户的操作，这里只是打印用户名和密码作为示例
    println!("添加用户：{}，密码：{}", user.username, user.password);

    // 返回响应
    (StatusCode::CREATED, Json(user))
}

pub async fn create_user(
    // this argument tells axum to parse the request body
    // as JSON into a `CreateUser` type
    Json(payload): Json<CreateUser>,
) -> impl IntoResponse {
    // insert your application logic here
    let user = User {
        id: 1337,
        username: payload.username,
    };

    let client_options = ClientOptions::parse("mongodb://localhost:27017").await.unwrap();
    let client = Client::with_options(client_options).unwrap();

    let db_name = "puti";
    let coll_name = "user";

    insert_user(&client, db_name, coll_name, user).await;

    // this will be converted into a JSON response
    // with a status code of `201 Created`
    (StatusCode::CREATED)
}


// the input to our `create_user` handler
#[derive(Deserialize)]
pub struct CreateUser {
    pub username: String,
}

pub fn training() {
    println!("In training");
}