use chrono::{DateTime, Utc};
use mongodb::bson::oid::ObjectId;
use serde::Serialize;

#[derive(Serialize)]
pub struct GenericResponse {
    pub status: String,
    pub message: String,
}

#[allow(non_snake_case)]
#[derive(Serialize, Debug)]
pub struct UserResponse {
    pub sys_id: Option<ObjectId>,
    pub id: String,
    pub username: String,
    pub nickname: String,
    pub password: String,
    pub avatar: String,
    pub email: String,
    pub is_delete: Option<bool>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Serialize, Debug)]
pub struct UserData {
    pub user: UserResponse,
}

#[derive(Serialize, Debug)]
pub struct SingleUserResponse {
    pub status: &'static str,
    pub data: UserData,
}

#[derive(Serialize, Debug)]
pub struct UserListResponse {
    pub status: &'static str,
    pub results: usize,
    pub users: Vec<UserResponse>,
}

#[derive(Serialize, Debug)]
pub struct TokenMessageResponse {
    pub code : u8,
    pub token : String,
    pub message : String,
}

#[derive(Serialize, Debug)]
pub struct MessageResponse {
    pub code : u8,
    pub status : String,
    pub message : String,
}
