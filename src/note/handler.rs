use std::{collections::HashMap, sync::Arc};
use tokio::{self, fs, io::AsyncWriteExt};
use axum::{
    extract::{Path, Query, State, Multipart, Request},
    http::StatusCode,
    body::Bytes,
    response::{Html, Redirect},
    response::IntoResponse,
    response::Response,
    BoxError, Json,
};
use mongodb::bson::Array;
use tokio_util::io::StreamReader;
use crate::{
    error::MyError,
    note::schema::{CreateNoteSchema, FilterOptions, UpdateNoteSchema},
    AppState,
};
use serde::{Deserialize, Serialize};
use futures::{Stream, TryStreamExt};
use tokio::{fs::File, io::BufWriter};
use std::io;

const UPLOADS_DIRECTORY: &str = "img";
#[derive(Deserialize)]
pub struct SubjectArgsOpt {
    pub page: Option<i32>,
    pub keyword: Option<String>,
}

pub async fn health_checker_handler(
    Query(args): Query<HashMap<String, String>>
) -> impl IntoResponse {
    let a = format!("{:?}", args);
    println!("{:?}", args);
    // let page = args.page.unwrap_or(0);
    // let keyword = args.keyword.unwrap_or("".to_string());
    // format!("Page {}, keyword: {} of subjects", page, keyword);
    // println!("page {}",page);
    // println!("keyword {}",keyword);

    const MESSAGE: &str = "RESTful API in Rust using Axum Framework and MongoDB";

    let json_response = serde_json::json!({
        "status": "success",
        "message": MESSAGE
    });

    Json(json_response)
}


pub async fn show_form() -> Html<&'static str> {
    Html(
        r#"
        <!doctype html>
        <html>
            <head></head>
            <body>
                <form action="/" method="post" enctype="multipart/form-data">
                    <label>
                        Upload file:
                        <input type="file" name="file" multiple>
                    </label>

                    <input type="submit" value="Upload files">
                </form>
            </body>
        </html>
        "#,
    )
}

pub async fn stream_show_form() -> Html<&'static str> {
    Html(
        r#"
        <!doctype html>
        <html>
            <head>
                <title>Upload something!</title>
            </head>
            <body>
                <form action="/" method="post" enctype="multipart/form-data">
                    <div>
                        <label>
                            Upload file:
                            <input type="file" name="file" multiple>
                        </label>
                    </div>

                    <div>
                        <input type="submit" value="Upload files">
                    </div>
                </form>
            </body>
        </html>
        "#,
    )
}
pub async fn accept_form(
    mut multipart: Multipart
) {
    while let Some(mut field) = multipart.next_field().await.unwrap() {
        // let name = field.name().unwrap().to_string();
        // let mut o_file = fs::File::create(&name).await.unwrap();
        let filename: String = field.file_name().unwrap().to_string();

        let img_folder = std::path::Path::new("img");
        let file_path = img_folder.join(&filename);
        let mut o_file = fs::File::create(&file_path).await.unwrap();
       
        while let Ok(chun_data) =  field.chunk().await{
            if let Some(bytes_data) = chun_data {
                o_file.write_all(&bytes_data).await.unwrap();
            } else {
                break;
            }
        }

        let content_type = field.content_type().unwrap().to_string();
        let data = field.bytes().await.unwrap();
        let image_url = format!("/img/{filename}");


        println!(
            "Length of (`{filename}`: `{content_type}`) is {} bytes",
            data.len()
        );
    }
}

// pub async fn serve_image(Path(filename): Path<String>) -> Response<BoxBody> {
//     let img_folder = std::path::Path::new("img");
//     let file_path = img_folder.join(&filename);

//     if let Ok(file) = fs::File::open(&file_path).await {
//         Response::builder()
//             .header("Content-Type", "image/jpeg") // or appropriate content type
//             .body(BoxBody::from(file))
//             .unwrap()
//     } else {
//         Response::builder()
//             .status(StatusCode::NOT_FOUND)
//             .body(BoxBody::default())
//             .unwrap()
//     }
// }

pub async fn show_img(
    Path(filename): Path<String>,
) -> String {
    let img_str = format!("img/{}", filename);
    println!("img_str: {}", img_str);
    std::fs::read_to_string(img_str).unwrap()
}

pub async fn note_list_handler(
    opts: Option<Query<FilterOptions>>,
    State(app_state): State<Arc<AppState>>,
) -> Result<impl IntoResponse, (StatusCode, Json<serde_json::Value>)> {
    let Query(opts) = opts.unwrap_or_default();

    let limit = opts.limit.unwrap_or(10) as i64;
    let page = opts.page.unwrap_or(1) as i64;

    match app_state
        .db
        .fetch_notes(limit, page)
        .await
        .map_err(MyError::from)
    {
        Ok(res) => Ok(Json(res)),
        Err(e) => Err(e.into()),
    }
}

pub async fn save_request_body(
    Path(file_name): Path<String>,
    request: Request,
) -> Result<(), (StatusCode, String)> {
    stream_to_file(&file_name, request.into_body().into_data_stream()).await
}

// Handler that accepts a multipart form upload and streams each field to a file.
pub async fn stream_accept_form(mut multipart: Multipart) -> Result<Redirect, (StatusCode, String)> {
    while let Ok(Some(field)) = multipart.next_field().await {
        let file_name = if let Some(file_name) = field.file_name() {
            println!("file_name: {}", file_name);
            file_name.to_owned()
        } else {
            continue;
        };

        stream_to_file(&file_name, field).await?;
    }

    Ok(Redirect::to("/"))
}

// Save a `Stream` to a file
async fn stream_to_file<S, E>(path: &str, stream: S) -> Result<(), (StatusCode, String)>
where
    S: Stream<Item = Result<Bytes, E>>,
    E: Into<BoxError>,
{
    if !path_is_valid(path) {
        return Err((StatusCode::BAD_REQUEST, "Invalid path".to_owned()));
    }

    async {
        // Convert the stream into an `AsyncRead`.
        let body_with_io_error = stream.map_err(|err| io::Error::new(io::ErrorKind::Other, err));
        let body_reader = StreamReader::new(body_with_io_error);
        futures::pin_mut!(body_reader);

        // Create the file. `File` implements `AsyncWrite`.
        let path = std::path::Path::new(UPLOADS_DIRECTORY).join(path);
        let mut file = BufWriter::new(File::create(path).await?);

        // Copy the body into the file.
        tokio::io::copy(&mut body_reader, &mut file).await?;

        Ok::<_, io::Error>(())
    }
    .await
    .map_err(|err| (StatusCode::INTERNAL_SERVER_ERROR, err.to_string()))
}

// to prevent directory traversal attacks we ensure the path consists of exactly one normal
// component
fn path_is_valid(path: &str) -> bool {
    let path = std::path::Path::new(path);
    let mut components = path.components().peekable();

    if let Some(first) = components.peek() {
        if !matches!(first, std::path::Component::Normal(_)) {
            return false;
        }
    }

    components.count() == 1
}

pub async fn create_note_handler(
    State(app_state): State<Arc<AppState>>,
    Json(body): Json<CreateNoteSchema>,
) -> Result<impl IntoResponse, (StatusCode, Json<serde_json::Value>)> {
    match app_state.db.create_note(&body).await.map_err(MyError::from) {
        Ok(res) => Ok((StatusCode::CREATED, Json(res))),
        Err(e) => Err(e.into()),
    }
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

pub async fn edit_note_handler(
    Path(id): Path<String>,
    State(app_state): State<Arc<AppState>>,
    Json(body): Json<UpdateNoteSchema>,
) -> Result<impl IntoResponse, (StatusCode, Json<serde_json::Value>)> {
    match app_state
        .db
        .edit_note(&id, &body)
        .await
        .map_err(MyError::from)
    {
        Ok(res) => Ok(Json(res)),
        Err(e) => Err(e.into()),
    }
}

pub async fn delete_note_handler(
    Path(id): Path<String>,
    State(app_state): State<Arc<AppState>>,
) -> Result<impl IntoResponse, (StatusCode, Json<serde_json::Value>)> {
    match app_state.db.delete_note(&id).await.map_err(MyError::from) {
        Ok(_) => Ok(StatusCode::NO_CONTENT),
        Err(e) => Err(e.into()),
    }
}
