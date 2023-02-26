use crate::handlers::api::manager::websockets::messages::ws_handler;
use crate::handlers::api::user::create::create_user;
use crate::handlers::api::user::direct_message;
use crate::handlers::api::user::direct_message::get::get_direct_messages;
use crate::handlers::api::user::direct_message::message::get::get_message_dm;
use crate::handlers::api::user::direct_message::message::post::post_message_dm;
use crate::handlers::api::user::login::user_login;
use crate::handlers::api::user::selfinfo::get_info_of_self;
use crate::handlers::api::user::token::verify;
use crate::handlers::default;
use crate::ServerState;
use axum::Router;

/// All routes related to the users
pub fn user_routes() -> Router<ServerState> {
    axum::Router::new()
        .route("/create", axum::routing::post(create_user))
        .route("/update", axum::routing::put(default))
        .route("/delete", axum::routing::delete(default))
        .route("/logout", axum::routing::get(default))
        .route("/websocket", axum::routing::get(ws_handler))
        .route("/login", axum::routing::post(user_login))
        .route("/token/verify", axum::routing::get(verify))
        .route("/me", axum::routing::get(get_info_of_self))
        .nest("/direct_message", direct_message())
}

pub fn direct_message() -> Router<ServerState> {
    axum::Router::new()
        .route("/", axum::routing::get(get_direct_messages).post(default))
        .nest(
            "/create/:id",
            Router::new().route(
                "/",
                axum::routing::get(direct_message::post::create_direct_message),
            ),
        )
        .nest(
            "/:id",
            Router::new()
                .route("/message", axum::routing::get(get_message_dm).post(default))
                .route("/message/:message_id", axum::routing::get(post_message_dm))
                .route("/users", axum::routing::get(default)),
        )
}
