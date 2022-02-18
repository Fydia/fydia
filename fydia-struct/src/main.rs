use fydia_struct::{
    channel::{DirectMessage, DirectMessageInner, ParentId},
    roles::ChannelAccess,
    server::ServerId,
};

pub fn main() {
    println!(
        "{}",
        serde_json::to_string(&ParentId::DirectMessage(DirectMessage {
            users: DirectMessageInner::Users(Vec::new())
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
        serde_json::to_string(&ChannelAccess(Vec::new())).unwrap_or_default()
    );
}
