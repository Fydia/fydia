use axum::Router;

use crate::handlers::default;

/// All routes related to the roles
pub fn roles_routes() -> Router {
    axum::Router::new()
        .route("/create", axum::routing::get(default))
        .nest(
            "/:id",
            axum::Router::new()
                .route("/color", axum::routing::get(default).post(default))
                .route("/name", axum::routing::get(default).post(default))
                .route("/description", axum::routing::get(default))
                .nest(
                    "/channelaccess",
                    axum::Router::new()
                        .route("/add/:channelid", axum::routing::post(default))
                        .route("/remove/:channelid", axum::routing::post(default)),
                ),
        )
}
