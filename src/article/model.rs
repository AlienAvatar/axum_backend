use chrono::prelude::*;
use mongodb::bson::{self, oid::ObjectId};
use serde::{Deserialize, Serialize};

#[allow(non_snake_case)]
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct ArticleModel {
    #[serde(rename = "_id")]
    pub sys_id: ObjectId,
    pub id: String,
    pub author: String,
    pub title: String,
    pub content: String,
    pub support_users: Vec<String>,
    pub support_count: i32,
    pub views_count: i32,
    pub category: String,
    pub cover_img: String,
    pub is_delete: Option<bool>,
    #[serde(with = "bson::serde_helpers::chrono_datetime_as_bson_datetime")]
    pub created_at: DateTime<Utc>,
    #[serde(with = "bson::serde_helpers::chrono_datetime_as_bson_datetime")]
    pub updated_at: DateTime<Utc>,
}

#[allow(non_snake_case)]
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct UpdateArticleModel {
    pub title: String,
    pub author: String,
    pub content: String,
    pub category: String,
    pub support_count: i32,
    pub support_users: Vec<String>,
    pub views_count: i32,
    pub cover_img: String,
}