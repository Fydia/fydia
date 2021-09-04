use std::panic::RefUnwindSafe;

use crate::handlers::federation::{event_handler, send_test_message};
use gotham::router::builder::*;
use gotham::{pipeline::chain::PipelineHandleChain, router::builder::ScopeBuilder};

/// All routes related to the fedaration
pub fn federation_routes<C, P>(router: &mut ScopeBuilder<C, P>)
where
    C: PipelineHandleChain<P> + Copy + Send + Sync + 'static,
    P: RefUnwindSafe + Send + Sync + 'static,
{
    router.post("/event/send").to_async(event_handler);
    router.get("/test").to_async(send_test_message);
}
