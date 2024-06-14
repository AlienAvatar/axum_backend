use chrono::prelude::*;
use mongodb::bson::{self, oid::ObjectId};
use serde::{Deserialize, Serialize};

#[allow(non_snake_case)]
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct CommentModel {
    #[serde(rename = "_id")]
    pub sys_id: ObjectId,
    pub id: String,
    pub article_id: String,
    pub author: String,
    pub content: String,
    pub good_count: i32,
    pub is_delete: Option<bool>,
    #[serde(with = "bson::serde_helpers::chrono_datetime_as_bson_datetime")]
    pub created_at: DateTime<Utc>,
    #[serde(with = "bson::serde_helpers::chrono_datetime_as_bson_datetime")]
    pub updated_at: DateTime<Utc>,
}

#[allow(non_snake_case)]
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct UpdateCommentModel {
    pub id: String,
    pub article_id: String,
    pub author: String,
    pub content: String,
    pub good_count: i32,
    pub is_delete: Option<bool>,
    #[serde(with = "bson::serde_helpers::chrono_datetime_as_bson_datetime")]
    pub updated_at: DateTime<Utc>,
}
