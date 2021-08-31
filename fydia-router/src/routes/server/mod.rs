use std::panic::RefUnwindSafe;

use fydia_struct::pathextractor::{ChannelExtractor, ServerExtractor};
use gotham::{pipeline::chain::PipelineHandleChain, router::builder::ScopeBuilder};

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
use gotham::router::builder::*;

/// All routes related to the server
pub fn server_routes<C, P>(router: &mut ScopeBuilder<C, P>)
where
    C: PipelineHandleChain<P> + Copy + Send + Sync + 'static,
    P: RefUnwindSafe + Send + Sync + 'static,
{
    router.get("").to_async(get_server_of_user);
    router.post("/create").to_async(create_server);
    router
        .get("/join/:serverid")
        .with_path_extractor::<ServerExtractor>()
        .to_async(join);
    router.scope("/:serverid", |router| {
        router
            .get("")
            .with_path_extractor::<ServerExtractor>()
            .to_async(get_server);
        router.scope("/channel", |router| {
            router
                .post("/create")
                .with_path_extractor::<ServerExtractor>()
                .to_async(create_channel);
            router.scope("/:channelid", channelid);
        });
        router.scope("/roles", roles_routes);
    });
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

pub fn channelid<C, P>(router: &mut ScopeBuilder<C, P>)
where
    C: PipelineHandleChain<P> + Copy + Send + Sync + 'static,
    P: RefUnwindSafe + Send + Sync + 'static,
{
    router
        .get("")
        .with_path_extractor::<ChannelExtractor>()
        .to_async(info_channel);

    router
        .delete("")
        .with_path_extractor::<ChannelExtractor>()
        .to_async(delete_channel);

    router
        .put("name")
        .with_path_extractor::<ChannelExtractor>()
        .to_async(update_name);

    router
        .put("description")
        .with_path_extractor::<ChannelExtractor>()
        .to_async(update_description);

    router.scope("/messages", |router| {
        router
            .get("")
            .with_path_extractor::<ChannelExtractor>()
            .to_async(get_message);
        router
            .post("")
            .with_path_extractor::<ChannelExtractor>()
            .to_async(post_messages);
    });

    router.scope("/join", |router| {
        router
            .get("")
            .with_path_extractor::<ChannelExtractor>()
            .to_async(join_channel);
    });
}
