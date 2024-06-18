use std::sync::Arc;

use axum::{
    extract::{Path, Query, State},
    http::StatusCode,
    response::IntoResponse,
    Json,
};

use crate::{
    error::MyError,
    note::schema::{CreateNoteSchema, FilterOptions, UpdateNoteSchema},
    AppState,
};
use super::schema::ParamOptions;



pub async fn health_checker_handler(
    id: Option<ParamOptions>,
) -> impl IntoResponse {

    dbg!(id);
    const MESSAGE: &str = "RESTful API in Rust using Axum Framework and MongoDB";

    let json_response = serde_json::json!({
        "status": "success",
        "message": MESSAGE
    });

    Json(json_response)
}

pub async fn get_note_handler(
    Path(id): Path<String>,
    State(app_state): State<Arc<AppState>>,
) -> Result<impl IntoResponse, (StatusCode, Json<serde_json::Value>)> {
    match app_state.db.get_note(&id).await.map_err(MyError::from) {
        Ok(res) => Ok(Json(res)),
        Err(e) => Err(e.into()),
    }
}


