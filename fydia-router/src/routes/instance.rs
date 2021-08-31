use std::panic::RefUnwindSafe;

use crate::handlers::api::instance::public_key::public_key;
use crate::handlers::default;
use gotham::handler::HandlerResult;
use gotham::helpers::http::response::create_response;
use gotham::router::builder::*;
use gotham::state::State;
use gotham::{pipeline::chain::PipelineHandleChain, router::builder::ScopeBuilder};
use reqwest::StatusCode;

/// All routes related to the instances
pub fn instance_routes<C, P>(router: &mut ScopeBuilder<C, P>)
where
    C: PipelineHandleChain<P> + Copy + Send + Sync + 'static,
    P: RefUnwindSafe + Send + Sync + 'static,
{
    router.get("/public_key").to_async(public_key);
    router.get("/version").to(default);
}

/// Handler return version
pub async fn version(state: State) -> HandlerResult {
    let res = create_response(
        &state,
        StatusCode::OK,
        mime::TEXT_PLAIN_UTF_8,
        env!("CARGO_PKG_VERSION"),
    );
    Ok((state, res))
}
