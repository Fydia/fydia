//! Router of fydia
//! Working with gotham.rs
#![deny(clippy::unwrap_used)]
#![deny(clippy::expect_used)]
#![warn(missing_debug_implementations)]

/// Handling routes
pub mod handlers;
/// All router routes
pub mod routes;

#[macro_use]
extern crate gotham_derive;

#[macro_use]
extern crate logger;

use std::process::exit;

use crate::handlers::api::websocket::Websockets;
use crate::handlers::default;
use crate::routes::federation::federation_routes;
use crate::routes::instance::instance_routes;
use crate::routes::server::server_routes;
use crate::routes::user::user_routes;
use fydia_config::Config;
use fydia_crypto::key::private_to_public;
use fydia_sql::connection::get_connection;
use fydia_sql::setup::create_tables;
use fydia_sql::sqlpool::{Repo, SqlPool};
use fydia_struct::instance::{Instance, RsaData};
use gotham::handler::HandlerResult;
use gotham::hyper::{Body, Response};
use gotham::middleware::state::StateMiddleware;
use gotham::pipeline::new_pipeline;
use gotham::pipeline::single::single_pipeline;
use gotham::router::builder::*;
use gotham::router::builder::{build_router, DrawRoutes};
use gotham::router::Router;
use gotham::state::State;
/// Return gotham's router
pub async fn get_router(config: Config) -> Router {
    info!(format!("Fydia - {}", env!("CARGO_PKG_VERSION")));
    info!("Waiting database");
    let database = Repo::new(get_connection(&config.database).await);
    success!("Database connected");
    info!("Info init database");
    if let Err(e) = create_tables(&database.get_pool()).await {
        error!(format!("Error: {}", e.to_string()));
        exit(0);
    }
    success!("Init successfully");
    info!("Try to generate RSA keys");
    let privatekey = match fydia_crypto::key::generate::generate_key() {
        Ok(key) => {
            success!("RSA keys are successfully generated");
            key
        }
        Err(_) => {
            error!("Can't generate RSA keys");
            exit(0);
        }
    };

    info!("try to get ip adress of the server");
    let domain = if config.instance.domain.is_empty() {
        if let Ok(req) = reqwest::Client::new()
            .get("http://ifconfig.io")
            .header("User-Agent", "curl/7.55.1")
            .send()
            .await
        {
            if let Ok(text) = req.text().await {
                text
            } else {
                panic!("Domain is not valid")
            }
        } else {
            panic!("Domain is not valid")
        }
    } else {
        config.instance.domain.clone()
    };

    success!(format!("Ip is : {}", domain));
    info!(format!("Listen on: http://{}", config.format_ip()));
    let public_key = if let Some(public_key) = private_to_public(privatekey.clone()) {
        public_key
    } else {
        panic!("Public key error");
    };

    let (chain, pipelineset) = single_pipeline(
        new_pipeline()
            .add(StateMiddleware::new(SqlPool::new(database)))
            .add(StateMiddleware::new(Websockets::new()))
            .add(StateMiddleware::new(RsaData(
                privatekey.clone(),
                public_key,
            )))
            .add(StateMiddleware::new(Instance::new(
                fydia_struct::instance::Protocol::HTTP,
                domain,
                config.server.port as u16,
            )))
            .build(),
    );

    build_router(chain, pipelineset, |router| {
        router.get("/").to_async(get_client);
        //router.get("/json").to_async(json);
        router.scope("/api", |router| {
            router.get("/").to(default);
            router.scope("/instance", instance_routes);
            router.scope("/user", user_routes);
            router.scope("/server", server_routes);
            router.scope("/federation", federation_routes);
        });
    })
}
const INDEX: &str = include_str!("../index.html");
/// Return index client
pub async fn get_client(state: State) -> HandlerResult {
    Ok((state, Response::new(Body::from(INDEX))))
}
