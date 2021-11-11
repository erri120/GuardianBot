use std::{
    collections::HashMap,
    sync::{Arc},
};
use ahash::{RandomState};
use serenity::model::channel::{ChannelType, PartialChannel};
use tokio::sync::RwLock;
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
    pub included_channels: Vec<ChannelSetting>
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
            included_channels: Vec::new()
        }
    }
}

impl GuildSetting {
    pub fn reset(&mut self) {
        self.active = false;
        self.max_repeats = 3;
        self.include_all_channels = true;
        self.excluded_channels = Vec::new();
        self.included_channels = Vec::new();
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