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
use crate::routes::federation::federation_routes;
use crate::routes::instance::instance_routes;
use crate::routes::server::server_routes;
use crate::routes::user::user_routes;
use axum::body::Body;
use axum::handler::Handler;
use axum::response::IntoResponse;
use axum::{AddExtensionLayer, Router};
use client::client_router;
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
    info!(format!(
        "Fydia - {}({})",
        env!("CARGO_PKG_VERSION"),
        env!("GIT_HASH")
    ));
    get_axum_router(
        get_database_connection(&config.database).await,
        &config.instance,
        &config.format_ip(),
        config.server.port as u16,
    )
    .await
}

pub async fn get_axum_router(
    database: DbConnection,
    instance: &InstanceConfig,
    formated_ip: &str,
    port: u16,
) -> axum::Router {
    #[cfg(not(test))]
    #[cfg(debug_assertions)]
    if let Err(error) = fydia_sql::samples::insert_samples(&database).await {
        error!(error);
    }

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
    let public_key = if let Some(public_key) = private_to_public(&privatekey) {
        public_key
    } else {
        panic!("Public key error");
    };

    let websocket_manager =
        Arc::new(crate::handlers::api::manager::websockets::manager::WbManager::spawn().await);
    let typing_manager =
        Arc::new(crate::handlers::api::manager::typing::TypingManager::spawn().await);
    if let Err(error) = typing_manager.set_websocketmanager(&websocket_manager) {
        error!(error);
        exit(1);
    };
    if let Err(error) = typing_manager.set_selfmanager(&typing_manager) {
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
        .nest("/", client_router())
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
            latency.as_millis(),
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

#[cfg(not(feature = "flutter_client"))]
mod client {
    use axum::{response::Html, routing, Router};

    /// Return index client
    pub async fn client() -> Html<String> {
        Html(INDEX.to_string())
    }

    pub fn client_router() -> Router {
        Router::new().route("/", routing::get(client))
    }

    const INDEX: &str = include_str!("../index.html");
}

#[cfg(feature = "flutter_client")]
mod client {
    //! Flutter fydia_client need
    //!     - manifest.json
    //!     - favicon.png (optional)
    //!     - icons/
    //!     - main.dart.js
    //!     - assets/

    use axum::{
        body::{boxed, Full},
        extract::Path,
        response::{Html, IntoResponse, Response},
        routing, Router,
    };
    use http::{header, StatusCode};
    use rust_embed::RustEmbed;

    /// Return index client
    pub async fn client() -> Html<String> {
        Html(INDEX.to_string())
    }

    pub async fn maindartjs() -> Response {
        Response::builder()
            .header("Content-Type", "application/javascript")
            .body(boxed(Full::from(
                include_bytes!("../fydiapackages/fydiaclient/build/web/main.dart.js").to_vec(),
            )))
            .unwrap()
    }

    pub async fn manifestjson() -> Response {
        Response::builder()
            .header("Content-Type", "application/json")
            .body(boxed(Full::from(
                include_bytes!("../fydiapackages/fydiaclient/build/web/manifest.json").to_vec(),
            )))
            .unwrap()
    }

    pub async fn flutterserverworking() -> Response {
        Response::builder()
            .header("Content-Type", "application/javascript")
            .body(boxed(Full::from(
                include_bytes!("../fydiapackages/fydiaclient/build/web/flutter_service_worker.js")
                    .to_vec(),
            )))
            .unwrap()
    }

    pub async fn versionjson() -> Response {
        Response::builder()
            .header("Content-Type", "application/json")
            .body(boxed(Full::from(
                include_bytes!("../fydiapackages/fydiaclient/build/web/version.json").to_vec(),
            )))
            .unwrap()
    }

    pub fn client_router() -> Router {
        Router::new()
            .route("/", routing::get(client))
            .route("/main.dart.js", routing::get(maindartjs))
            .route("/manifest.json", routing::get(manifestjson))
            .route(
                "/flutter_service_worker.js",
                routing::get(flutterserverworking),
            )
            .route("/version.json", routing::get(versionjson))
            .route("/assets/*path", routing::get(get_asset))
    }

    #[derive(RustEmbed)]
    #[folder = "fydiapackages/fydiaclient/build/web/assets"]
    struct Asset;

    pub struct StaticFile<T>(pub T);

    impl<T> IntoResponse for StaticFile<T>
    where
        T: Into<String>,
    {
        fn into_response(self) -> Response {
            let path = self.0.into();

            match Asset::get(path.as_str()) {
                Some(content) => {
                    let body = boxed(Full::from(content.data));
                    let mime = mime_guess::from_path(path).first_or_octet_stream();
                    Response::builder()
                        .header(header::CONTENT_TYPE, mime.as_ref())
                        .body(body)
                        .unwrap()
                }
                None => Response::builder()
                    .status(StatusCode::NOT_FOUND)
                    .body(boxed(Full::from("404")))
                    .unwrap(),
            }
        }
    }

    pub async fn get_asset(Path(path): Path<String>) -> impl IntoResponse {
        StaticFile(path.strip_prefix("/").unwrap().to_string())
    }

    const INDEX: &str = include_str!("../fydiapackages/fydiaclient/build/web/index.html");
}
