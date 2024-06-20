use serde::{Deserialize, Serialize};
use chrono::{DateTime, Utc};
use mongodb::bson::{self};
#[derive(Deserialize, Debug, Default)]
pub struct FilterOptions {
    pub page: Option<usize>,
    pub limit: Option<usize>,
    pub id: Option<String>,
    pub nickname: Option<String>,
    pub username: Option<String>,
    pub is_delete: Option<bool>,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct CreateUserSchema {
    pub username: String,
    pub nickname: String,
    pub password: String,
    pub avatar: String,
    pub email: String,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct UpdateUserSchema {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub email: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub nickname: Option<String>,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct UpdateUserPasswordSchema {
    //暂时不能更改password
    pub password: String,
}
#[derive(Serialize, Deserialize, Debug)]
pub struct VaildUserSchema {
    pub username: String,
    pub password: String,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct DeleteUserSchema {
    pub is_delete : Option<bool>,
    #[serde(with = "bson::serde_helpers::chrono_datetime_as_bson_datetime")]
    pub updated_at: DateTime<Utc>,
}
