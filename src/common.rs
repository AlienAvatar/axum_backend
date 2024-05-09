use serde::{Deserialize, Serialize};
use axum::response::IntoResponse;
use axum::response::ErrorResponse;

#[derive(Deserialize, Serialize)]
pub struct Resp<T> where T: Serialize {
    code: i32,
    message: String,
    data: Option<T>,
}

impl<T: Serialize> Resp<T> {
    pub fn ok(data: T) -> Self {
        Resp { code: 0, message: "ok".to_owned(), data: Some(data) }
    }

    // pub fn to_json_result(&self) -> Result<IntoResponse, ErrorResponse> {
    //     Ok(IntoResponse::Ok().json(self))
    // }
}

impl Resp<()> {
    pub fn err(error: i32, message: &str) -> Self {
        Resp { code: error, message: message.to_owned(), data: None }
    }
}