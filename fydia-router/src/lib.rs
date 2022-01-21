//! Router of fydia
//! Working with gotham.rs
#![deny(clippy::unwrap_used)]
#![deny(clippy::expect_used)]
#![warn(missing_debug_implementations)]

/// Handling routes
pub mod handlers;
/// All router routes
pub mod routes;
/// All tests of Router
pub mod tests;

#[macro_use]
extern crate logger;
use crate::handlers::api::manager::typing::TypingManagerChannelTrait;
use crate::handlers::api::manager::websockets::test_message;
use crate::routes::federation::federation_routes;
use crate::routes::instance::instance_routes;
use crate::routes::server::server_routes;
use crate::routes::user::user_routes;
use axum::body::Body;
use axum::handler::Handler;
use axum::response::{Html, IntoResponse};
use axum::{AddExtensionLayer, Router};
use fydia_config::{Config, DatabaseConfig, InstanceConfig};
use fydia_crypto::key::private_to_public;
use fydia_sql::connection::get_connection;
use fydia_sql::setup::create_tables;
use fydia_sql::sqlpool::DbConnection;
use fydia_struct::instance::{Instance, RsaData};
use handlers::api::manager::typing::TypingManagerChannel;
use handlers::api::manager::websockets::manager::WebsocketManagerChannel;
use http::Response;
use std::process::exit;
use std::sync::Arc;
use std::time::Duration;
use tower::ServiceBuilder;
use tower_http::trace::{OnRequest, OnResponse, TraceLayer};
use tracing::Span;

pub async fn get_database_connection(config: &DatabaseConfig) -> DbConnection {
    info!("Waiting database");
    let database = Arc::new(get_connection(config).await) as DbConnection;
    success!("Database connected");
    info!("Init database");
    if let Err(e) = create_tables(&database).await {
        error!(format!("Error: {}", e));
        exit(0);
    }
    success!("Init successfully");

    database
}

pub async fn get_axum_router_from_config(config: Config) -> axum::Router {
    get_axum_router(
        get_database_connection(&config.database).await,
        &config.instance,
        &config.format_ip(),
        *&config.server.port as u16,
    )
    .await
}

pub async fn get_axum_router(
    database: DbConnection,
    instance: &InstanceConfig,
    formated_ip: &String,
    port: u16,
) -> axum::Router {
    info!(format!(
        "Fydia - {}({})",
        env!("CARGO_PKG_VERSION"),
        env!("GIT_HASH")
    ));

    #[cfg(not(test))]
    #[cfg(debug_assertions)]
    fydia_sql::samples::insert_samples(&database).await;

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
    success!(format!("Ip is : {}", instance.domain));
    info!(format!("Listen on: http://{}", formated_ip));
    let public_key = if let Some(public_key) = private_to_public(privatekey.clone()) {
        public_key
    } else {
        panic!("Public key error");
    };

    let websocket_manager =
        Arc::new(crate::handlers::api::manager::websockets::manager::WbManager::spawn().await);
    let typing_manager =
        Arc::new(crate::handlers::api::manager::typing::TypingManager::spawn().await);
    if let Err(error) = typing_manager.set_websocketmanager(websocket_manager.clone()) {
        error!(error);
        exit(1);
    };
    if let Err(error) = typing_manager.set_selfmanager(typing_manager.clone()) {
        error!(error);
        exit(1)
    }

    get_router(
        database,
        Arc::new(Instance::new(
            fydia_struct::instance::Protocol::HTTP,
            &instance.domain,
            port,
        )),
        Arc::new(RsaData(privatekey, public_key)),
        websocket_manager,
        typing_manager,
    )
}

pub fn get_router(
    database: DbConnection,
    instance: Arc<Instance>,
    rsadata: Arc<RsaData>,
    websocket_manager: Arc<WebsocketManagerChannel>,
    typing_manager: Arc<TypingManagerChannel>,
) -> Router {
    axum::Router::new()
        .route("/", axum::routing::get(client))
        .route("/test", axum::routing::get(test_message))
        .nest(
            "/api",
            axum::Router::new()
                .nest("/instance", instance_routes())
                .nest("/user", user_routes())
                .nest("/server", server_routes())
                .nest("/federation", federation_routes()),
        )
        .fallback(not_found.into_service())
        .layer(AddExtensionLayer::new(database))
        .layer(AddExtensionLayer::new(instance))
        .layer(AddExtensionLayer::new(rsadata))
        .layer(AddExtensionLayer::new(websocket_manager))
        .layer(AddExtensionLayer::new(typing_manager))
        .layer(
            ServiceBuilder::new()
                .layer(TraceLayer::new_for_http().on_request(Log).on_response(Log)),
        )
}

#[derive(Clone)]
struct Log;

impl OnRequest<Body> for Log {
    fn on_request(&mut self, request: &http::Request<Body>, _: &tracing::Span) {
        logger::info!(format!("{} {}", request.method(), request.uri()));
    }
}

impl<B> OnResponse<B> for Log {
    fn on_response(self, response: &Response<B>, latency: Duration, _: &Span) {
        logger::info!(format!(
            "({}ms) => {}",
            latency.as_micros(),
            response.status(),
        ));
    }
}

/// Return index client
async fn not_found() -> impl IntoResponse {
    (
        http::StatusCode::NOT_FOUND,
        String::from("Route Not Found : 404"),
    )
}

/// Return index client
pub async fn client() -> Html<String> {
    Html(INDEX.to_string())
}

const INDEX: &str = include_str!("../index.html");
