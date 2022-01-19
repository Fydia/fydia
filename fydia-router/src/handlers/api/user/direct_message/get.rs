use axum::extract::Extension;
use axum::response::IntoResponse;
use fydia_sql::impls::channel::SqlDirectMessages;

use fydia_sql::sqlpool::DbConnection;
use fydia_struct::channel::ParentId;
use fydia_struct::{channel::DirectMessage, response::FydiaResponse};
use http::HeaderMap;

use crate::handlers::basic::BasicValues;

pub async fn get_direct_messages(
    headers: HeaderMap,
    Extension(database): Extension<DbConnection>,
) -> impl IntoResponse {
    let user = match BasicValues::get_user(&headers, &database).await {
        Ok(v) => v,
        Err(string) => {
            return FydiaResponse::new_error(string);
        }
    };

    match DirectMessage::get_by_userid(&database, user.id).await {
        Ok(mut channels) => {
            for i in channels.iter_mut() {
                if let ParentId::DirectMessage(direct_message) = &mut i.parent_id {
                    if let Err(e) = direct_message.userid_to_user(&database).await {
                        error!(e);
                    };
                }
            }

            FydiaResponse::new_ok_json(channels)
        }
        Err(e) => {
            error!(e);
            FydiaResponse::new_error("Error")
        }
    }
}
