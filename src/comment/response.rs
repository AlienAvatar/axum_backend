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
    pub id: ObjectId,
    pub comment_id: String,
    pub content: String,
    pub article_num: String,
    pub author: String,
    pub good_count: i32,
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
    pub message : String,
}