use serde::{Deserialize, Serialize};
use chrono::{DateTime, Utc};
use mongodb::bson::{self};

#[derive(Deserialize, Debug, Default)]
pub struct FilterOptions {
    pub page: Option<usize>,
    pub limit: Option<usize>,
}

#[derive(Deserialize, Debug)]
pub struct ParamOptions {
    pub id: String,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct CreateCommentSchema {
    pub article_id: String,
    pub author: String,
    pub content: String,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct GetCommentsByArticleNumSchema {
    pub article_id: String,
}


#[derive(Serialize, Deserialize, Debug)]
pub struct UpdateCommentSchema {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub article_id: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub content: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub author: Option<String>,
    pub good_count: Option<i32>,
}


#[derive(Serialize, Deserialize, Debug)]
pub struct DeleteCommentSchema {
    pub is_delete : Option<bool>,
    #[serde(with = "bson::serde_helpers::chrono_datetime_as_bson_datetime")]
    pub updated_at: DateTime<Utc>,
}
