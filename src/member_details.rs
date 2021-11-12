use std::{
    collections::HashMap,
    sync::{Arc},
};
use ahash::{RandomState};
use serenity::model::channel::Message;
use tokio::sync::RwLock;
use serenity::prelude::TypeMapKey;

pub struct MemberDetails;

impl TypeMapKey for MemberDetails {
    type Value = Arc<RwLock<HashMap<u64, MemberInfo, RandomState>>>;
}

pub struct MemberInfo {
    pub last_mentions: Vec<MessageInfo>
    // pub last_messages: Vec<MessageInfo>
}

pub struct MessageInfo {
    pub timestamp: i64,
    pub channel_id: u64,
    pub guild_id: u64
}

impl Default for MemberInfo {
    fn default() -> Self {
        return MemberInfo {
            // last_messages: Vec::new()
            last_mentions: Vec::new()
        }
    }
}

impl From<&serenity::model::channel::Message> for MessageInfo {
    fn from(discord_message: &Message) -> Self {
        return MessageInfo {
            timestamp: discord_message.timestamp.timestamp(),
            channel_id: discord_message.channel_id.0,
            guild_id: discord_message.guild_id.expect("Message does not have a GuildId!").0
        }
    }
}
