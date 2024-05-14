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
        create_user_handler, get_user_by_username_handler, user_list_handler, login_user_handler, 
        update_user_handler, delete_user_handler, logout_user_handler
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
        // user
        .route("/api/user/list/", get(user_list_handler))
        .route("/api/user/create/", post(create_user_handler))
        .route("/api/user/get/:username", get(get_user_by_username_handler))
        .route("/api/user/login/", post(login_user_handler))
        .route("/api/user/logout/", post(logout_user_handler))
        .route("/api/user/update/:username", post(update_user_handler))
        .route("/api/user/delete/:username", post(delete_user_handler))
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


