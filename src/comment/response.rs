use chrono::{DateTime, Utc};
use mongodb::bson::oid::ObjectId;
use serde::Serialize;

#[derive(Serialize, Debug)]
pub struct CommentListResponse {
    pub status: &'static str,
    pub results: usize,
    pub comments: Vec<CommentResponse>,
}


#[allow(non_snake_case)]
#[derive(Serialize, Debug)]
pub struct CommentResponse {
    pub sys_id: ObjectId,
    pub id: String,
    pub content: String,
    pub article_id: String,
    pub author: String,
    pub support_count: i32,
    pub support_users: Vec<String>,
    pub is_delete: Option<bool>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Serialize, Debug)]
pub struct CommentData {
    pub comment: CommentResponse,
}


#[derive(Serialize, Debug)]
pub struct SingleCommentResponse {
    pub status: &'static str,
    pub data: CommentData,
}

#[derive(Serialize, Debug)]
pub struct MessageResponse {
    pub code : u8,
    pub status : String,
    pub message : String,
}