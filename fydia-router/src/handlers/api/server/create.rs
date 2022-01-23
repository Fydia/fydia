use axum::body::Bytes;
use axum::extract::Extension;
use axum::response::IntoResponse;
use fydia_sql::impls::server::SqlServer;
use fydia_sql::sqlpool::DbConnection;
use fydia_struct::response::FydiaResponse;
use fydia_struct::server::Server;

use http::HeaderMap;
use serde_json::Value;

use crate::handlers::basic::BasicValues;

pub async fn create_server(
    headers: HeaderMap,
    body: Bytes,
    Extension(database): Extension<DbConnection>,
) -> impl IntoResponse {
    let mut user = match BasicValues::get_user(&headers, &database).await {
        Ok(v) => v,
        Err(error) => return FydiaResponse::new_error(error),
    };

    if let Ok(body) = String::from_utf8(body.to_vec()) {
        if let Ok(value) = serde_json::from_str::<Value>(body.as_str()) {
            if let Some(name) = value.get("name") {
                if let Some(name_str) = name.as_str() {
                    let mut server = Server::new(name_str, user.id.clone());
                    match server.insert_server(&database).await {
                        Ok(_) => match server.join(&mut user, &database).await {
                            Ok(_) => return FydiaResponse::new_ok(server.id.id),
                            Err(e) => {
                                error!(e);
                                return FydiaResponse::new_error("Cannot join the server");
                            }
                        },
                        Err(e) => {
                            error!(e);
                            return FydiaResponse::new_error("Cannot join the server");
                        }
                    }
                }
            }
        }
    }

    FydiaResponse::new_error("Json Error")
}
