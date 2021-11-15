mod guild_settings;
mod member_details;
mod commands;
mod message_handler;

use std::{
    collections::{HashMap},
    sync::Arc
};

use anyhow::{anyhow, Context};

use serenity::{
    async_trait,
    model::{
        gateway::Ready,
        id::{
            GuildId
        },
        interactions::{
            application_command::{
                ApplicationCommand,
                ApplicationCommandInteraction,
                ApplicationCommandInteractionDataOptionValue
            },
            Interaction,
            InteractionResponseType,
        },
        guild::Guild,
        channel::{
            Message,
            MessageType,
            GuildChannel
        }
    },
    prelude::*,
};
use crate::commands::create_commands;

use crate::guild_settings::{ChannelSetting, GuildSettings};
use crate::member_details::{MemberDetails, MessageInfo};
use crate::message_handler::is_spam;

struct Handler;

impl Handler {
    async fn interaction_create_with_result(&self, ctx: &serenity::prelude::Context, command: &ApplicationCommandInteraction) -> Result<String, anyhow::Error> {
        let guild_id = command.guild_id
            .with_context(|| format!("Interaction {} does not have a guild_id!", command.id.0))?;

        let guild = guild_id.to_guild_cached(&ctx.cache).await
            .with_context(|| format!("Unable to get guild {}", guild_id))?;

        let member = command.member.as_ref()
            .with_context(|| format!("Interaction {} does not have a member!", command.id.0))?;

        let permissions = guild.member_permissions(&ctx.http, member.user.id).await
            .with_context(|| format!("Unable to get permissions of member '{}' ({}) in guild '{}' ({})", member.user.name, member.user.id, guild.name, guild.id.0))?;

        if !permissions.administrator() {
            // TODO: report this
            return Ok("You do not have permissions to use this command!".to_string());
        }

        let data_read = ctx.data.read().await;
        let guild_settings_lock = data_read.get::<GuildSettings>()
            .with_context(|| format!("Unable to get GuildSettings from TypeMap!"))?
            .clone();

        // TODO: maybe put this lock somewhere else, not sure how to handle this
        // also need to investigate how bad the performance is, having a lock on every message & command is not good
        // probably best to make this a read-only lock and make it a write lock if we need to
        let mut guild_settings = guild_settings_lock.write().await;
        let current_guild_settings = guild_settings.entry(guild_id.0).or_default();

        // TODO: cleanup, also: https://github.com/serenity-rs/serenity/issues/1462
        // TODO: report changes
        let command_name = command.data.name.as_str();
        if command_name == "reset" {
            current_guild_settings.reset();
            return Ok("The settings for this Guild have been reset, make sure to re-activate the bot".to_string());
        } else if command_name == "activate" {
            return if current_guild_settings.active {
                Err(anyhow!("The bot is already active for this Guild!"))
            } else {
                current_guild_settings.active = true;
                Ok("The bot is now activated for this Guild".to_string())
            }
        } else if command_name == "deactivate" {
            return if current_guild_settings.active {
                current_guild_settings.active = false;
                Ok("The bot is now deactivated for this Guild".to_string())
            } else {
                Err(anyhow!("The bot is already deactivated for this Guild!"))
            }
        } else if command_name == "exclude_all" {
            return if current_guild_settings.include_all_channels {
                current_guild_settings.include_all_channels = false;
                Ok("All channels are now excluded, you can still manually include channels you want monitored".to_string())
            } else {
                Err(anyhow!("All channels are already excluded!"))
            }
        } else if command_name == "include_all" {
            return if current_guild_settings.include_all_channels {
                Err(anyhow!("All channels are already included!"))
            } else {
                current_guild_settings.include_all_channels = true;
                Ok("All channels are now included, you can still manually exclude channels you don't want to be monitored".to_string())
            }
        } else if command_name == "exclude_channel" {
            let channel_option = command
                .data.options.get(0)
                .with_context(|| format!("Unable to get option"))?
                .resolved.as_ref()
                .with_context(|| format!("Unable to resolve option"))?;

            return if let ApplicationCommandInteractionDataOptionValue::Channel(channel) = channel_option {
                return if current_guild_settings.include_all_channels {
                    if current_guild_settings.excluded_channels.iter().any(|i| i.id == channel.id.0) {
                        return Err(anyhow!("The channel <#{}> ({}) is already excluded!", channel.id.0, channel.id.0));
                    } else {
                        current_guild_settings.excluded_channels.push(ChannelSetting::from(channel));
                    }

                    Ok(format!("The channel <#{}> ({}) will now be excluded", channel.id.0, channel.id.0))
                } else {
                    // we are excluding every channel, in this mode the command can remove a channel from the included_channels list
                    let channel_index = current_guild_settings.included_channels.iter().position(|x| x.id == channel.id.0);
                    return match channel_index {
                        Some(i) => {
                            current_guild_settings.included_channels.remove(i);
                            Ok(format!("Channel <#{}> ({}) has been removed from the included channels list", channel.id.0, channel.id.0))
                        },
                        None => Err(anyhow!("The channel <#{}> ({}) has not been included so it can't be excluded!", channel.id.0, channel.id.0))
                    }
                }
            } else {
                Err(anyhow!("Invalid channel!"))
            }
        } else if command_name == "include_channel" {
            let channel_option = command
                .data.options.get(0)
                .with_context(|| format!("Unable to get option"))?
                .resolved.as_ref()
                .with_context(|| format!("Unable to resolve option"))?;

            return if let ApplicationCommandInteractionDataOptionValue::Channel(channel) = channel_option {
                return if current_guild_settings.include_all_channels {
                    // we are include every channel, in this mode the command can remove a channel from the excluded_channels list
                    let channel_index = current_guild_settings.excluded_channels.iter().position(|x| x.id == channel.id.0);
                    return match channel_index {
                        Some(i) => {
                            current_guild_settings.excluded_channels.remove(i);
                            Ok(format!("Channel <#{}> ({}) has been removed from the excluded channels list", channel.id.0, channel.id.0))
                        },
                        None => Err(anyhow!("The channel <#{}> ({}) has not been excluded so it can't be included!", channel.id.0, channel.id.0))
                    }
                } else {
                    if current_guild_settings.included_channels.iter().any(|i| i.id == channel.id.0) {
                        return Err(anyhow!("The channel <#{}> ({}) is already included!", channel.id.0, channel.id.0));
                    } else {
                        current_guild_settings.included_channels.push(ChannelSetting::from(channel));
                    }

                    Ok(format!("The channel <#{}> ({}) will now be included", channel.id.0, channel.id.0))
                }
            } else {
                Err(anyhow!("Invalid channel!"))
            }
        } else if command_name == "set_log_channel" {
            let channel_option = command
                .data.options.get(0)
                .with_context(|| format!("Unable to get option"))?
                .resolved.as_ref()
                .with_context(|| format!("Unable to resolve option"))?;

            return if let ApplicationCommandInteractionDataOptionValue::Channel(channel) = channel_option {
                current_guild_settings.log_channel = Some(channel.id.0);
                Ok(format!("The channel <#{}> ({}) is now the log channel of this guild", channel.id.0, channel.id.0))
            } else {
                Err(anyhow!("Invalid channel!"))
            }
        } else {
            Err(anyhow!("Unknown command!"))
        }
    }

    async fn message_with_result(&self, ctx: &serenity::client::Context, new_message: &Message) -> Result<(), anyhow::Error> {
        // ignore bots (includes this bot, had a funny loop where the bot replied to itself forever and banned itself)
        if new_message.author.bot {
            return Ok(());
        }

        if new_message.kind != MessageType::Regular && new_message.kind != MessageType::InlineReply {
            return Ok(());
        }

        // TODO: figure out the best order for this
        // should this check if the message is spam or if the guild settings are valid (active & channel is monitored)?
        // in theory this bot shouldn't get messages from guilds that are not active, no point in having a deactivated bot
        // so that check might is not important and could go last as it it least likely to be false
        // this leaves checking if the message is spam and if the channel is included or not

        if !is_spam(new_message) {
            return Ok(());
        }

        let guild = new_message.guild(&ctx.cache).await
            .with_context(|| format!("Unable to get guild of message {}", new_message.id.0))?;
        let guild_id = guild.id;

        // accessing global data
        let data_read = ctx.data.read().await;

        // getting the current guild settings
        let guild_settings_lock = data_read.get::<GuildSettings>()
            .with_context(|| format!("Unable to get GuildSettings from TypeMap!"))?
            .clone();

        // guild settings with shared read access
        let guild_settings = guild_settings_lock.read().await;
        let current_guild_settings = guild_settings.get(&guild_id.0)
            .with_context(|| format!("Unable to find Guild {} in HashMap!", guild_id.0))?;

        if !current_guild_settings.active {
            return Ok(());
        }

        let channel = guild.channels.get(&new_message.channel_id)
            .with_context(|| format!("Unable to get channel of message {}", new_message.id.0))?;

        let ignore_channel = match channel.category_id {
            Some(category) => current_guild_settings.should_ignore_channel(category.0) || current_guild_settings.should_ignore_channel(channel.id.0),
            None => current_guild_settings.should_ignore_channel(channel.id.0)
        };

        if ignore_channel {
            return Ok(());
        }

        // getting the current member info
        let member_details_lock = data_read.get::<MemberDetails>()
            .with_context(|| format!("Unable to get MemberDetails from TypeMap!"))?
            .clone();

        let should_ban;
        {
            // member info with exclusive write access
            let mut member_details = member_details_lock.write().await;
            let current_member_info = member_details.entry(new_message.author.id.0).or_default();

            // add current mention message to vector
            current_member_info.last_mentions.push(MessageInfo::from(new_message));

            // ban user
            should_ban = current_member_info.last_mentions.len() >= current_guild_settings.max_repeats as usize;
        }

        if should_ban {
            // TODO: cross-server ban
            new_message.reply_mention(&ctx.http, format!("has reached the limit and will be banned for spamming.")).await
                .with_context(|| format!("Unable to reply to message!"))?;

            guild_id.ban_with_reason(&ctx.http, new_message.author.id, 1, format!("Guardian Ban: Spamming")).await
                .with_context(|| format!("Unable to ban user!"))?;

            current_guild_settings.send_log_message(&ctx.http, &guild, format!("The user <@{}> ({}) has been banned for spamming.", new_message.author.id.0, new_message.author.id.0)).await?;
        } else {
            new_message.reply_ping(&ctx.http, format!("You do not have the permission to mention everyone and will be banned if you continue.")).await
                .with_context(|| format!("Unable to reply to message!"))?;

            current_guild_settings.send_log_message(&ctx.http, &guild, format!("The user <@{}> ({}) has sent a message mentioning everyone without permissions in channel <#{}>:\n```txt\n{}```", new_message.author.id.0, new_message.author.id.0, channel.id.0, new_message.content)).await?;
        }

        Ok(())
    }

    async fn channel_delete_with_result(&self, ctx: &serenity::client::Context, channel: &GuildChannel) -> Result<(), anyhow::Error> {
        let data_read = ctx.data.read().await;

        let guild_settings_lock = data_read.get::<GuildSettings>()
            .with_context(|| format!("Unable to get GuildSettings from TypeMap!"))?
            .clone();

        let mut guild_settings = guild_settings_lock.write().await;
        let mut current_guild_settings = guild_settings.get_mut(&channel.guild_id.0)
            .with_context(|| format!("Unable to find Guild {} in HashMap!", channel.guild_id.0))?;

        match current_guild_settings.log_channel {
            Some(log_channel) => {
                if log_channel == channel.id.0 {
                    println!("Log channel {} got removed in guild {}", channel.id.0, channel.guild_id.0);
                    current_guild_settings.log_channel = None;
                }
            }
            _ => {}
        }

        let included_channel_index = current_guild_settings.included_channels.iter().position(|x| x.id == channel.id.0);
        match included_channel_index {
            Some(i) => {
                println!("Removing channel {} from included_channels vec of guild {}", channel.id.0, channel.guild_id.0);
                current_guild_settings.included_channels.remove(i);
            },
            _ => {}
        }

        let excluded_channel_index = current_guild_settings.excluded_channels.iter().position(|x| x.id == channel.id.0);
        match excluded_channel_index {
            Some(i) => {
                println!("Removing channel {} from excluded_channels vec of guild {}", channel.id.0, channel.guild_id.0);
                current_guild_settings.excluded_channels.remove(i);
            },
            _ => {}
        }

        Ok(())
    }
}

#[async_trait]
impl EventHandler for Handler {
    async fn channel_delete(&self, ctx: serenity::client::Context, channel: &GuildChannel) {
        match self.channel_delete_with_result(&ctx, channel).await {
            Ok(()) => return,
            Err(why) => println!("{}", why)
        }
    }

    async fn guild_create(&self, ctx: serenity::prelude::Context, guild: Guild, is_new: bool) {
        if is_new {
            println!("Bot got added to Guild '{}' ({})", guild.name, guild.id.0);
        } else {
            println!("Bot connected to Guild '{}' ({})", guild.name, guild.id.0);
        }

        let data_read = ctx.data.read().await;
        let guild_settings_lock = data_read.get::<GuildSettings>()
            .expect("Unable to get GuildSettings from TypeMap!")
            .clone();

        let mut guild_settings = guild_settings_lock.write().await;
        let _ = guild_settings.entry(guild.id.0).or_default();
    }

    async fn message(&self, ctx: serenity::client::Context, new_message: Message) {
        match self.message_with_result(&ctx, &new_message).await {
            Ok(()) => return,
            Err(why) => println!("{}", why)
        }
    }

    async fn ready(&self, ctx: serenity::prelude::Context, ready: Ready) {
        println!("{} is connected!", ready.user.name);

        let test_guild_id = std::env::var("GUILD_ID");
        match test_guild_id {
            Ok(x) => {
                let test_guild_id: u64 = x.parse().expect("Unable to parse environment variable \"GUILD_ID\" as u64!");
                let test_guild = GuildId(test_guild_id);

                println!("Creating commands for guild {}", test_guild_id);
                test_guild.set_application_commands(&ctx.http, create_commands).await.unwrap();
            },
            Err(_) => {
                println!("Creating global commands");
                ApplicationCommand::set_global_application_commands(&ctx.http, create_commands).await.unwrap();
            }
        }
    }

    async fn interaction_create(&self, ctx: serenity::prelude::Context, interaction: Interaction) {
        if let Interaction::ApplicationCommand(command) = interaction {
            let res = self.interaction_create_with_result(&ctx, &command).await;

            if let Err(why) = command.create_interaction_response(&ctx.http, |response| {
                response
                    .kind(InteractionResponseType::ChannelMessageWithSource)
                    .interaction_response_data(|message| message
                        .content(match res {
                            Ok(x) => x,
                            Err(why) => {
                                println!("{}", why);
                                format!("{}", why)
                            }
                        }))
            }).await
            {
                println!("Unable to response to slash command: {}", why);
                return;
            } else {
                return;
            }
        }
    }
}

#[tokio::main]
async fn main() {
    let token = std::env::var("DISCORD_TOKEN")
        .expect("Missing environment variable: \"DISCORD_TOKEN\"");

    let application_id: u64 = std::env::var("APPLICATION_ID")
        .expect("Missing environment variable: \"APPLICATION_ID\"")
        .parse()
        .expect("Unable to parse environment variable \"APPLICATION_ID\" as u64!");

    let mut client = Client::builder(&token)
        .event_handler(Handler)
        .application_id(application_id)
        .await.expect("Err creating client");

    {
        let mut data = client.data.write().await;
        data.insert::<GuildSettings>(Arc::new(RwLock::new(HashMap::default())));
        data.insert::<MemberDetails>(Arc::new(RwLock::new(HashMap::default())));
    }

    if let Err(why) = client.start().await {
        println!("Client error: {:?}", why);
    }
}
