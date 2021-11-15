use std::{
    collections::HashMap,
    sync::{Arc},
};

use ahash::{RandomState};
use anyhow::Context;
use tokio::sync::RwLock;

use serenity::model::{
    channel::{ChannelType, PartialChannel},
    guild::Guild,
    id::ChannelId
};
use serenity::http::Http;
use serenity::prelude::TypeMapKey;

pub struct GuildSettings;

impl TypeMapKey for GuildSettings {
    type Value = Arc<RwLock<HashMap<u64, GuildSetting, RandomState>>>;
}

#[derive(Debug)]
pub struct GuildSetting {
    pub active: bool,
    pub max_repeats: i32,
    pub include_all_channels: bool,
    pub excluded_channels: Vec<ChannelSetting>,
    pub included_channels: Vec<ChannelSetting>,
    pub log_channel: Option<u64>
}

#[derive(Debug)]
pub struct ChannelSetting {
    pub id: u64,
    pub kind: ChannelType
}

impl Default for GuildSetting {
    fn default() -> Self {
        return GuildSetting {
            active: false,
            max_repeats: 3,
            include_all_channels: true,
            excluded_channels: Vec::new(),
            included_channels: Vec::new(),
            log_channel: None
        }
    }
}

impl GuildSetting {
    pub fn reset(&mut self) {
        let default = GuildSetting::default();
        self.active = default.active;
        self.max_repeats = default.max_repeats;
        self.include_all_channels = default.include_all_channels;
        self.excluded_channels = default.excluded_channels;
        self.included_channels = default.included_channels;
        self.log_channel = default.log_channel;
    }

    pub fn should_ignore_channel(&self, id: u64) -> bool {
        if self.include_all_channels {
            // return if we are excluding the current channel in include-all mode
            if self.excluded_channels.iter().any(|x| x.id == id) {
                return true;
            }
        } else {
            // return if we are not including the current channel in exclude-all mode
            if !self.included_channels.iter().any(|x| x.id == id) {
                return true;
            }
        }

        return false;
    }

    pub async fn send_log_message(&self, http: impl AsRef<Http>, guild: &Guild, message: String) -> Result<(), anyhow::Error> {
        match self.log_channel {
            Some(log_channel) => {
                let log_channel = guild.channels.get(&ChannelId(log_channel))
                    .with_context(|| format!("Unable to get log channel {}", log_channel))?;

                log_channel.say(http, message).await
                    .with_context(|| format!("Unable to send message to log channel {}", log_channel))?;
            },
            _ => {}
        }

        return Ok(())
    }
}

impl From<&PartialChannel> for ChannelSetting {
    fn from(other: &PartialChannel) -> Self {
        return ChannelSetting {
            id: other.id.0,
            kind: other.kind
        }
    }
}
