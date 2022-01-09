use axum::body::Body;
use axum::extract::{Extension, Path};
use axum::http::Request;
use axum::response::IntoResponse;
use fydia_sql::impls::server::SqlServer;
use fydia_sql::impls::user::SqlUser;
use fydia_sql::sqlpool::DbConnection;
use fydia_struct::response::FydiaResponse;
use fydia_struct::server::{Server, ServerId};
use fydia_struct::user::{Token, User};

use crate::new_response;

pub async fn join(
    request: Request<Body>,
    Path(server_id): Path<String>,
    Extension(database): Extension<DbConnection>,
) -> impl IntoResponse {
    let headers = request.headers();
    let mut res = new_response();
    let token = if let Some(token) = Token::from_headervalue(headers) {
        token
    } else {
        FydiaResponse::new_error("Token Error").update_response(&mut res);
        return res;
    };

    if let Some(mut user) = User::get_user_by_token(&token, &database).await {
        if let Ok(mut server) =
            Server::get_server_by_id(ServerId::new(server_id.clone()), &database).await
        {
            if user.servers.is_join(ServerId::new(server_id)) {
                FydiaResponse::new_error("Already join").update_response(&mut res);
            } else if let Err(error) = server.join(&mut user, &database).await {
                FydiaResponse::new_error("Cannot join").update_response(&mut res);
                error!(error);
            };
        }
    }

    res
}
