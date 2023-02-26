use axum::Router;

use crate::{handlers::default, ServerState};

/// All routes related to the roles
pub fn roles_routes() -> Router<ServerState> {
    axum::Router::new()
        .route("/create", axum::routing::get(default))
        .nest(
            "/:id",
            axum::Router::new()
                .route("/color", axum::routing::get(default).post(default))
                .route("/name", axum::routing::get(default).post(default))
                .route("/description", axum::routing::get(default)),
        )
}
