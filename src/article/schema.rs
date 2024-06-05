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
pub struct CreateArticleSchema {
    pub title: String,
    pub author: String,
    pub content: String,
    pub category: String,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct UpdateArticleSchema {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub password: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub email: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub author: Option<String>,
    pub is_delete : Option<bool>,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct DeleteArticleSchema {
    pub is_delete : Option<bool>,
    #[serde(with = "bson::serde_helpers::chrono_datetime_as_bson_datetime")]
    pub updated_at: DateTime<Utc>,
}
