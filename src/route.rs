use std::sync::Arc;

use axum::{
    routing::{get, post},
    Router,
};

use crate::{
    note::handler::{
        create_note_handler, delete_note_handler, edit_note_handler, get_note_handler,
        health_checker_handler, note_list_handler,
    }, 
    user::handler::{
        user_note_list_handler,
    }, 
    AppState
};

pub fn create_router(app_state: Arc<AppState>) -> Router {
    Router::new()
        .route("/api/healthchecker", get(health_checker_handler))
        .route("/api/notes/", post(create_note_handler))
        .route("/api/notes/list", get(note_list_handler))
        .route(
            "/api/notes/:id",
            get(get_note_handler)
                .patch(edit_note_handler)
                .delete(delete_note_handler),
        )
        .route("/api/login/getuserlist", get(user_note_list_handler))
        .with_state(app_state)

    // let app = Router::new()
    //     // login
    //     .route("/login/getUserList", get(GetUserList))
    //     .route("/login/addUser", post(CreateUser))
    //     .route("/login/validUser", post(ValidUser))
    //     .route("/upload_page", get(upload_page))
    //     .route("/do_upload", post(do_upload));
    // //let app = Router::new().route("/", get(home));
}

pub fn user_router() -> Router {
    Router::new()
        .route("/getUser", get(health_checker_handler))
        // .route(
        //     "/todos/:id",
        //     get(get_todo).put(update_todo).delete(delete_todo),
        // )
}

