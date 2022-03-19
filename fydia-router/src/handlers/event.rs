use crate::handlers::api::manager::websockets::manager::WebsocketManagerChannel;
use fydia_sql::impls::server::{SqlMember, SqlServer, SqlServerId};
use fydia_sql::sqlpool::DbConnection;
use fydia_struct::event::Event;

use super::api::manager::websockets::manager::WbManagerChannelTrait;
pub async fn event_handler(
    event: Event,
    database: &DbConnection,
    wbsockets: &WebsocketManagerChannel,
) {
    let database = database;

    if let Ok(server) = event.server_id.get_server(database).await {
        if let Ok(members) = server.get_user(database).await {
            if let Ok(members) = members.to_userinfo(database).await {
                if wbsockets.send(&event, &members).await.is_err() {
                    error!("Error");
                };
            }
        }
    }
}
