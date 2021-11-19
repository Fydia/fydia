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
extern crate logger;
use crate::handlers::api::websocket::test_message;
use crate::handlers::api::websocket::WebsocketManager;
use crate::routes::federation::federation_routes;
use crate::routes::instance::instance_routes;
use crate::routes::server::server_routes;
use crate::routes::user::user_routes;
use axum::body::Body;
use axum::handler::Handler;
use axum::response::{Html, IntoResponse};
use axum::AddExtensionLayer;
use fydia_config::Config;
use fydia_crypto::key::private_to_public;
use fydia_sql::connection::get_connection;
use fydia_sql::setup::create_tables;
use fydia_sql::sqlpool::DbConnection;
use fydia_struct::instance::{Instance, RsaData};
use http::{HeaderMap, HeaderValue, StatusCode};
use std::process::exit;
use std::sync::Arc;
use tower::ServiceBuilder;
use tower_http::trace::{OnRequest, TraceLayer};

pub fn new_response() -> (StatusCode, HeaderMap<HeaderValue>, String) {
    (StatusCode::OK, HeaderMap::new(), String::new())
}

pub async fn get_axum_router(config: Config) -> axum::Router {
    info!(format!("Fydia - {}", env!("CARGO_PKG_VERSION")));
    info!("Waiting database");
    let database = Arc::new(get_connection(&config.database).await) as DbConnection;
    success!("Database connected");
    info!("Info init database");
    if let Err(e) = create_tables(&database).await {
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
    let domain = /*if config.instance.domain.is_empty() {
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
    };*/
    "127.0.0.1".to_string();

    success!(format!("Ip is : {}", domain));
    info!(format!("Listen on: http://{}", config.format_ip()));
    let public_key = if let Some(public_key) = private_to_public(privatekey.clone()) {
        public_key
    } else {
        panic!("Public key error");
    };
    let websocket_manager = WebsocketManager::spawn().await;

    /*spawn(async move {
        for _ in 0..5000 {
            thread
                .0
                .send(WbManagerMessage::Add(
                    User::default(),
                    unbounded_channel().0,
                ))
                .unwrap();
        }
        println!("Finished Added");
    });*/

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
        .layer(AddExtensionLayer::new(database as DbConnection))
        .layer(AddExtensionLayer::new(Arc::new(Instance::new(
            fydia_struct::instance::Protocol::HTTP,
            domain,
            config.server.port as u16,
        ))))
        .layer(AddExtensionLayer::new(RsaData(
            privatekey.clone(),
            public_key,
        )))
        .layer(AddExtensionLayer::new(websocket_manager))
        .layer(ServiceBuilder::new().layer(TraceLayer::new_for_http().on_request(Log)))
}

#[derive(Clone)]
struct Log;

impl OnRequest<Body> for Log {
    fn on_request(&mut self, request: &http::Request<Body>, _: &tracing::Span) {
        logger::info!(format!("{} {}", request.method(), request.uri()));
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
