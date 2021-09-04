use crate::handlers::api::websocket::Websockets;
use fydia_sql::impls::server::{SqlServer, SqlServerId};
use fydia_sql::sqlpool::SqlPool;
use fydia_struct::event::Event;
use gotham::state::{FromState, State};

pub async fn event_handler(event: Event, state: &mut State) {
    let database = SqlPool::borrow_from(state).get_pool();

    let websockets = Websockets::borrow_mut_from(state);
    if let Some(server) = event.server.get_server(&database).await {
        let users = server.get_user(&database).await;
        websockets.send(&event.clone(), users, None, None).await;
    }
}
