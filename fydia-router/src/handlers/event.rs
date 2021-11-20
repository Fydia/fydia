use crate::handlers::api::websocket::WebsocketManagerChannel;
use fydia_sql::impls::server::{SqlServer, SqlServerId};
use fydia_sql::sqlpool::DbConnection;
use fydia_struct::event::Event;
pub async fn event_handler(
    event: Event,
    database: &DbConnection,
    wbsockets: &WebsocketManagerChannel,
) {
    let database = database;

    if let Ok(server) = event.server.get_server(database).await {
        if let Ok(members) = server.get_user(database).await {
            if wbsockets
                .send(event.clone(), members.members, None, None)
                .await
                .is_err()
            {
                error!("Error");
            };
        }
    }
}
