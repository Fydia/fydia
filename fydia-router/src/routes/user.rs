use std::panic::RefUnwindSafe;

use crate::handlers::api::user::create::create_user;
use crate::handlers::api::user::login::user_login;
use crate::handlers::api::websocket::messages::ws_handler;
use crate::handlers::default;
use fydia_struct::querystring::QsToken;
use gotham::router::builder::*;
use gotham::{pipeline::chain::PipelineHandleChain, router::builder::ScopeBuilder};

/// All routes related to the users
pub fn user_routes<C, P>(router: &mut ScopeBuilder<C, P>)
where
    C: PipelineHandleChain<P> + Copy + Send + Sync + 'static,
    P: RefUnwindSafe + Send + Sync + 'static,
{
    router.post("/login").to_async(user_login);
    router
        .get("/websocket")
        .with_query_string_extractor::<QsToken>()
        .to_async(ws_handler);
    router.post("/create").to_async(create_user);
    router.put("/update").to(default);
    router.delete("/delete").to(default);
    router.get("/logout").to(default);
    router.get("/").to(default);
}
