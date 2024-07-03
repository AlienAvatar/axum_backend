use chrono::{DateTime, Utc};
use mongodb::bson::oid::ObjectId;
use serde::Serialize;

#[derive(Serialize, Debug)]
pub struct ArticleListResponse {
    pub status: &'static str,
    pub count: usize,
    pub articles: Vec<ArticleResponse>,
}


#[allow(non_snake_case)]
#[derive(Serialize, Debug)]
pub struct ArticleResponse {
    pub sys_id: ObjectId,
    pub id: String,
    pub author: String,
    pub title: String,
    pub content: String,
    pub support_count: i32,
    pub views_count: i32,
    pub category: String,
    pub cover_img: String,
    pub is_delete: Option<bool>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}


#[derive(Serialize, Debug)]
pub struct ArticleData {
    pub article: ArticleResponse,
}


#[derive(Serialize, Debug)]
pub struct SingleArticleResponse {
    pub status: &'static str,
    pub data: ArticleData,
}
