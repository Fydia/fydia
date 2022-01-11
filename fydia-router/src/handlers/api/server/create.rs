use axum::body::Body;
use axum::extract::{BodyStream, Extension};
use axum::http::Request;
use axum::response::IntoResponse;
use futures::StreamExt;
use fydia_sql::impls::server::SqlServer;
use fydia_sql::impls::user::SqlUser;

use fydia_sql::sqlpool::DbConnection;
use fydia_struct::response::FydiaResponse;
use fydia_struct::server::Server;
use fydia_struct::user::{Token, User};

use serde_json::Value;

use crate::new_response;

pub async fn create_server(
    request: Request<Body>,
    mut body: BodyStream,
    Extension(database): Extension<DbConnection>,
) -> impl IntoResponse {
    let headers = request.headers();
    let mut res = new_response();
    let token = if let Some(token) = Token::from_headervalue(headers) {
        token
    } else {
        return res;
    };

    while let Some(Ok(body_bytes)) = body.next().await {
        if let Ok(body) = String::from_utf8(body_bytes.to_vec()) {
            if let Some(mut user) = User::get_user_by_token(&token, &database).await {
                if let Ok(value) = serde_json::from_str::<Value>(body.as_str()) {
                    if let Some(name) = value.get("name") {
                        if let Some(name_str) = name.as_str() {
                            let mut server = Server::new(name_str.to_string(), user.id.clone());
                            match server.insert_server(&database).await {
                                Ok(_) => match server.join(&mut user, &database).await {
                                    Ok(_) => FydiaResponse::new_ok(server.shortid)
                                        .update_response(&mut res),
                                    Err(e) => {
                                        FydiaResponse::new_error("Cannot join the server")
                                            .update_response(&mut res);
                                        error!(e);
                                    }
                                },
                                Err(e) => {
                                    FydiaResponse::new_error("Cannot join the server")
                                        .update_response(&mut res);
                                    error!(e);
                                }
                            }
                        }
                    }
                }
            }
        }
    }

    res
}
