use std::sync::Arc;

use axum::{
    routing::{get, post},
    Router,
};

use crate::{
    article::handler::{
        article_list_handler, create_article_handler, get_article_by_num_handler,
        update_article_by_num_handler, delete_article_by_num_handler
    }, 
    note::handler::{
        create_note_handler, delete_note_handler, edit_note_handler, get_note_handler,
        health_checker_handler, note_list_handler,
    }, 
    user::handler::{
        create_user_handler, delete_user_handler, get_user_by_username_handler, login_user_handler, logout_user_handler, update_user_handler, user_list_handler
    },
    comment::handler::{
        create_comment_handler, comment_list_handler, comment_list_by_aritcle_num_handler, get_comment_by_comment_id_handler,
        delete_comment_by_comment_id_handler, update_comment_by_id_handler
    },
    AppState
};

pub fn create_router(app_state: Arc<AppState>) -> Router {
    Router::new()
        .route("/api/healthchecker", get(health_checker_handler))
        .route("/api/notes/", post(create_note_handler))
        .route("/api/notes/list/", get(note_list_handler))
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
        //article
        .route("/api/article/list/", get(article_list_handler))
        .route("/api/article/create/", post(create_article_handler))
        .route("/api/article/get/:num", get(get_article_by_num_handler))
        .route("/api/article/update/:num", post(update_article_by_num_handler))
        .route("/api/article/delete/:num", post(delete_article_by_num_handler))
        //comment
        .route("/api/comment/list/", get(comment_list_handler))
        .route("/api/comment/list/:author_num", get(comment_list_by_aritcle_num_handler))
        .route("/api/comment/create/", post(create_comment_handler))
        .route("/api/comment/get/:comment_id", get(get_comment_by_comment_id_handler))
        .route("/api/comment/delete/:comment_id", post(delete_comment_by_comment_id_handler))
        .route("/api/comment/update/:comment_id", post(update_comment_by_id_handler))
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


