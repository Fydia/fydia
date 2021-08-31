use std::panic::RefUnwindSafe;

use crate::handlers::default;
use gotham::router::builder::*;
use gotham::{pipeline::chain::PipelineHandleChain, router::builder::ScopeBuilder};
/// All routes related to the roles
pub fn roles_routes<C, P>(router: &mut ScopeBuilder<C, P>)
where
    C: PipelineHandleChain<P> + Copy + Send + Sync + 'static,
    P: RefUnwindSafe + Send + Sync + 'static,
{
    router.post("/create").to(default);
    router.get("/info").to(default);
    router.scope("/:id", |router| {
        router.get("/color").to(default);
        router.post("/color").to(default);
        router.get("/name").to(default);
        router.post("/name").to(default);
        router.scope("/channelaccess", |router| {
            router.post("/add/:channelid");
            router.post("/remove/:channelid");
        });
    });
}
