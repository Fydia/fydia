use fydia_struct::channel::ChannelId;
use fydia_struct::server::ServerId;
use fydia_struct::user::User;
use parking_lot::Mutex;
use std::collections::HashMap;

struct TypingStruct(Mutex<HashMap<User, HashMap<ServerId, Vec<ChannelId>>>>);

impl TypingStruct {
    pub fn new() -> Self {
        Self(Mutex::new(HashMap::new()))
    }
    pub fn get(&self, user: &User) {}
    pub fn insert(
        &self,
        user: &mut User,
        serverid: ServerId,
        channelid: ChannelId,
    ) -> Result<(), ()> {
        let mut user = user.clone();
        let mut hashmap = &self.0.lock();

        //user.drop_sensitive_information();
        //let mut hashmap_typing = HashMap::new();
        //hashmap_typing.insert(serverid.clone(), vec![channelid]);
        //hashmap.insert(user, hashmap_typing);

        Ok(())
    }
}
