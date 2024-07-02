use serde::{Deserialize, Serialize};
use chrono::{DateTime, Utc};
use mongodb::bson::{self};

#[derive(Deserialize, Debug, Default)]
pub struct FilterOptions {
    pub page: Option<usize>,
    pub limit: Option<usize>,
    pub id: Option<String>,
    pub title: Option<String>,
    pub author: Option<String>,
    pub tags: Option<String>,
    pub is_delete: Option<bool>,
}

#[derive(Deserialize, Debug)]
pub struct ParamOptions {
    pub id: String,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct CreateArticleSchema {
    pub title: String,
    pub author: String,
    pub content: String,
    pub category: String,
    pub cover_img: Option<String>,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct UpdateArticleSchema {
    pub title: String,
    pub author: String,
    pub content: String,
    pub category: String,
    pub support_count: i32,
    pub views_count: i32,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct DeleteArticleSchema {
    pub is_delete : Option<bool>,
    #[serde(with = "bson::serde_helpers::chrono_datetime_as_bson_datetime")]
    pub updated_at: DateTime<Utc>,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct CrawelSchema {
}
