use axum::Router;

use crate::handlers::{
    api::server::{
        channels::{
            create::create_channel,
            delete::delete_channel,
            info_channel,
            messages::{
                get::get_messages,
                messageid::{delete::delete_message, get::get_message, post::update_message},
                post::post_messages,
            },
            permission::{
                get_permission,
                role::get_permission_of_role,
                user::{get_permission_of_user, post_permission_of_user},
            },
            typing::{start_typing, stop_typing},
            update::{update_description, update_name},
            vocal::join_channel,
        },
        create::create_server,
        get_server,
        info::get_server_of_user,
        join::join,
        picture::{get_picture_of_server, post_picture_of_server},
    },
    default,
};

use super::roles::roles_routes;

/// All routes related to the server
pub fn server_routes() -> Router {
    axum::Router::new()
        .route("/", axum::routing::get(get_server_of_user))
        .route("/create", axum::routing::post(create_server))
        .route("/join/:serverid", axum::routing::get(join))
        .nest(
            "/:serverid",
            axum::Router::new()
                .route("/", axum::routing::get(get_server))
                .route(
                    "/picture",
                    axum::routing::get(get_picture_of_server).post(post_picture_of_server),
                )
                .nest("/channel", channelid())
                .nest("/roles", roles_routes()),
        )
}

/// ```ignore
/// ChannelId:
///     /api/server/:serverid/channel/:channelid/
///         - GET / -> Give info of channel
///         - DELETE / -> Delete channel
///         - PUT
///             - /name -> Update name of channel
///             - /description -> Update description of channel
///         - GET /messages -> Give message of channel
///         - POST /messages -> Post a message into channel
/// ```
pub fn channelid() -> Router {
    axum::Router::new()
        .route("/create", axum::routing::post(create_channel))
        .nest(
            "/:channelid",
            axum::Router::new()
                .route("/", axum::routing::get(info_channel).delete(delete_channel))
                .route("/name", axum::routing::put(update_name))
                .route("/description", axum::routing::put(update_description))
                .route("/join", axum::routing::get(join_channel))
                .route("/permissions", axum::routing::get(get_permission))
                .nest(
                    "/permission",
                    Router::new()
                        .route("/role/:roleid", axum::routing::get(get_permission_of_role))
                        .route(
                            "/user/:userid",
                            axum::routing::get(get_permission_of_user)
                                .post(post_permission_of_user),
                        ),
                )
                .nest(
                    "/typing",
                    Router::new()
                        .route("/start", axum::routing::post(start_typing))
                        .route("/stop", axum::routing::post(stop_typing)),
                )
                .nest(
                    "/messages",
                    Router::new()
                        .route("/", axum::routing::get(get_messages).post(post_messages))
                        .nest("/:messageid/", messageid()),
                ),
        )
}

/// ```ignore
/// MessageId Route:
///     /api/server/:serverid/channel/:channelid/messages/:messageid
///         - GET /
///         - POST /
///         - DELETE /
/// ```
pub fn messageid() -> Router {
    axum::Router::new()
        .route(
            "/",
            axum::routing::get(get_message)
                .post(update_message)
                .delete(delete_message),
        )
        .route("/reactions", axum::routing::post(default).delete(default))
}
