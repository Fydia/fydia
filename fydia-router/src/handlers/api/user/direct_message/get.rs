use axum::body::Body;
use axum::extract::Extension;
use axum::http::Request;
use axum::response::IntoResponse;
use fydia_sql::impls::{channel::SqlDirectMessages, token::SqlToken};

use fydia_sql::sqlpool::DbConnection;
use fydia_struct::channel::ParentId;
use fydia_struct::user::UserId;
use fydia_struct::{channel::DirectMessage, response::FydiaResponse, user::Token};

use crate::new_response;

pub async fn get_direct_messages(
    request: Request<Body>,
    Extension(database): Extension<DbConnection>,
) -> impl IntoResponse {
    let mut res = new_response();
    let headers = request.headers();
    if let Some(token) = Token::from_headervalue(headers) {
        if let Some(e) = token.get_user(&database).await {
            match DirectMessage::get_by_userid(&database, UserId::new(e.id)).await {
                Ok(mut channels) => {
                    for i in channels.iter_mut() {
                        if let ParentId::DirectMessage(direct_message) = &mut i.parent_id {
                            if let Err(e) = direct_message.userid_to_user(&database).await {
                                error!(e);
                            };
                        }
                    }

                    FydiaResponse::new_ok_json(channels).update_response(&mut res);
                }
                Err(e) => {
                    error!(e);
                    FydiaResponse::new_error("Error").update_response(&mut res);
                }
            }
        };
    }

    res
}
