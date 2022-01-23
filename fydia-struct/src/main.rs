use fydia_struct::{
    channel::{DirectMessage, DirectMessageValue, ParentId},
    server::ServerId, roles::ChannelAccess,
};

pub fn main() {
    println!(
        "{}",
        serde_json::to_string(&ParentId::DirectMessage(DirectMessage {
            users: DirectMessageValue::Users(Vec::new())
        }))
        .unwrap_or_default()
    );

    println!(
        "{}",
        serde_json::to_string(&ParentId::ServerId(ServerId::new(String::new())))
            .unwrap_or_default()
    );

    println!(
        "{}",
        serde_json::to_string(&ChannelAccess(Vec::new()))
            .unwrap_or_default()
    );
}
