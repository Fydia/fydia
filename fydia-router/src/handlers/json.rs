/*use fydia_sql::impls::channel::SqlChannel;
use fydia_sql::impls::message::SqlMessage;
use fydia_sql::impls::server::SqlServer;
use fydia_sql::impls::user::SqlUser;
use fydia_sql::sqlpool::SqlPool;
use fydia_struct::server::{Server, ServerId};
use fydia_struct::user::User;
use gotham::handler::HandlerResult;

use fydia_struct::channel::Channel;
use fydia_utils::{generate_string, verify_password};
use gotham::helpers::http::response::create_response;
use gotham::hyper::StatusCode;
use gotham::state::{FromState, State};

pub async fn json(state: State) -> HandlerResult {
    let data = SqlPool::borrow_from(&state);
    let result = data.get_pool();
    let mp = &result;

    Server::get_server_by_id(ServerId::new("1ENwYDlsoe".to_string()), mp).await;
    let mut user = User::get_user_by_id(24, mp).await.unwrap();
    user.update_name(generate_string(15), mp).await.unwrap();
    user.update_token(mp).await.unwrap();
    user.update_password(String::from("HelloGoodPassword"), mp)
        .await
        .unwrap();
    verify_password(String::from("HelloGoodPassword"), user.password.clone());
    fydia_struct::messages::Message::get_messages_by_user_id(1, mp)
        .await
        .unwrap();
    let channels =
        Channel::get_channels_by_server_id("1ENwYDlsoepW9HHZEmYxEl9KKRQFBD".to_string(), mp).await;
    let server = Server::get_server_by_id(ServerId::new("1ENwYDlsoe".to_string()), mp)
        .await
        .unwrap();
    match (
        serde_json::to_string(&channels),
        serde_json::to_string(&user),
        serde_json::to_string(&server),
    ) {
        (Ok(channels), Ok(users), Ok(servers)) => todo!(),
        _ => {}
    }
    let res = create_response(
        &state,
        StatusCode::OK,
        mime::TEXT_PLAIN_UTF_8,
        format!(
            "{}\n{}\n{}",
            serde_json::to_string(&channels).unwrap(),
            serde_json::to_string(&user).unwrap(),
            serde_json::to_string(&server).unwrap()
        ),
    );
    Ok((state, res))
}
*/
