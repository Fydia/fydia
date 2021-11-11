use axum::Router;

use crate::handlers::api::{
    messages::{get::get_message, post::post_messages},
    server::{
        channels::{
            create::create_channel,
            delete::delete_channel,
            info_channel,
            update::{update_description, update_name},
            vocal::join_channel,
        },
        create::create_server,
        get_server,
        info::get_server_of_user,
        join::join,
    },
};

use super::roles::roles_routes;

/// All routes related to the server
pub fn server_routes() -> Router {
    axum::Router::new()
        .route("/", axum::routing::get(get_server_of_user))
        .route("/create", axum::routing::get(create_server))
        .route("/join/:serverid", axum::routing::get(join))
        .nest(
            "/:serverid",
            axum::Router::new()
                .route("/", axum::routing::get(get_server))
                .nest(
                    "/channel",
                    axum::Router::new()
                        .route("/create", axum::routing::get(create_channel))
                        .nest("/:channelid", channelid()),
                )
                .nest("/roles", roles_routes()),
        )
}

/// ```
///ChannelId Routes
/// /api/server/:serverid/channel/:channelid/
///     - GET / -> Give info of channel
///     - DELETE / -> Delete channel
///     - PUT
///         - /name -> Update name of channel
///         - /description -> Update description of channel
///     - GET /messages -> Give message of channel
///     - POST /messages -> Post a message into channel
///```

pub fn channelid() -> Router {
    axum::Router::new()
        .route("/create", axum::routing::get(create_channel))
        .nest(
            "/:channelid",
            axum::Router::new()
                .route("/", axum::routing::get(info_channel).delete(delete_channel))
                .route("/name", axum::routing::get(update_name))
                .route("/description", axum::routing::get(update_description))
                .route("/join", axum::routing::get(join_channel))
                .nest(
                    "/messages",
                    axum::Router::new()
                        .route("/", axum::routing::get(get_message).post(post_messages)),
                ),
        )
}
