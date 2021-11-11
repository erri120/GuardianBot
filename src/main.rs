mod guild_settings;

use std::{
    collections::{HashMap},
    sync::Arc
};

use anyhow::{anyhow, Context};

use serenity::{
    async_trait,
    model::{
        gateway::Ready,
        id::GuildId,
        interactions::{
            application_command::{
                ApplicationCommandInteraction,
                ApplicationCommandInteractionDataOptionValue,
                ApplicationCommandOptionType,
            },
            Interaction,
            InteractionResponseType,
        },
        guild::Guild,
        channel::Message
    },
    prelude::*,
};

use crate::guild_settings::{ChannelSetting, GuildSettings};

struct Handler;

impl Handler {
    async fn interaction_create_with_result(&self, ctx: &serenity::prelude::Context, command: &ApplicationCommandInteraction) -> Result<String, anyhow::Error> {
        let guild_id = command.guild_id
            .with_context(|| format!("Interaction {} does not have a guild_id!", command.id.0))?;

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
            return if current_guild_settings.include_all_channels {
                let channel_option = command
                    .data.options.get(0)
                    .with_context(|| format!("Unable to get option"))?
                    .resolved.as_ref()
                    .with_context(|| format!("Unable to resolve option"))?;

                return if let ApplicationCommandInteractionDataOptionValue::Channel(channel) = channel_option {
                    if current_guild_settings.excluded_channels.iter().any(|i| i.id == channel.id.0) {
                        return Err(anyhow!("The channel '{}' ({}) is already excluded!", channel.name, channel.id.0));
                    } else {
                        current_guild_settings.excluded_channels.push(ChannelSetting::from(channel));
                    }

                    Ok(format!("The channel '{}' ({}) will now be excluded", channel.name, channel.id.0))
                } else {
                    Err(anyhow!("Invalid channel!"))
                }
            } else {
                Err(anyhow!("Guild is in exclude-all-channels mode, change the settings to include all channels and call this command again! There is no point in excluding a certain channel if the bot is already excluding everything."))
            }
        } else if command_name == "include_channel" {
            return if current_guild_settings.include_all_channels {
                Err(anyhow!("Guild is in include-all-channels mode, change the settings to exclude all channels and call this command again! There is no point in including a certain channel if the bot is already including everything."))
            } else {
                let channel_option = command
                    .data.options.get(0)
                    .with_context(|| format!("Unable to get option"))?
                    .resolved.as_ref()
                    .with_context(|| format!("Unable to resolve option"))?;

                return if let ApplicationCommandInteractionDataOptionValue::Channel(channel) = channel_option {
                    if current_guild_settings.included_channels.iter().any(|i| i.id == channel.id.0) {
                        return Err(anyhow!("The channel '{}' ({}) is already included!", channel.name, channel.id.0));
                    } else {
                        current_guild_settings.included_channels.push(ChannelSetting::from(channel));
                    }

                    Ok(format!("The channel '{}' ({}) will now be included", channel.name, channel.id.0))
                } else {
                    Err(anyhow!("Invalid channel!"))
                }
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

        let guild_id = new_message.guild_id
            .with_context(|| format!("Message {} does not have a guild_id!", new_message.id.0))?;

        let data_read = ctx.data.read().await;
        let guild_settings_lock = data_read.get::<GuildSettings>()
            .with_context(|| format!("Unable to get GuildSettings from TypeMap!"))?
            .clone();

        let guild_settings = guild_settings_lock.read().await;
        let current_guild_settings = guild_settings.get(&guild_id.0)
            .with_context(|| format!("Unable to find Guild {} in HashMap!", guild_id.0))?;

        if !current_guild_settings.active {
            return Ok(());
        }

        let channel = new_message.channel_id.to_channel(&ctx.http).await
            .with_context(|| format!("Unable to get channel of message {}", new_message.id.0))?;

        // TODO: categories are also channels, need to somehow check if the channel is in a category which we include/exclude

        if current_guild_settings.include_all_channels {
            // we are included all channels and have check if we exclude the current one
            if current_guild_settings.excluded_channels.iter().any(|i| i.id == channel.id().0) {
                return Ok(());
            }

            new_message.reply_mention(&ctx.http, format!("include all channels -> not excluded")).await
                .with_context(|| format!("Unable to reply to message {}", new_message.id.0))?;
        } else {
            // we are excluding all channels and have to check if we are including the current one
            if current_guild_settings.included_channels.iter().any(|i| i.id == channel.id().0) {
                new_message.reply_mention(&ctx.http, format!("exclude all channels -> included")).await
                    .with_context(|| format!("Unable to reply to message {}", new_message.id.0))?;
            } else {
                return Ok(());
            }
        }

        Ok(())
    }
}

#[async_trait]
impl EventHandler for Handler {
    async fn guild_create(&self, _ctx: serenity::prelude::Context, guild: Guild, is_new: bool) {
        if is_new {
            println!("Bot got added to Guild '{}' ({})", guild.name, guild.id.0);
        } else {
            println!("Bot connected to Guild '{}' ({})", guild.name, guild.id.0);
        }
    }

    async fn message(&self, ctx: serenity::client::Context, new_message: Message) {
        match self.message_with_result(&ctx, &new_message).await {
            Ok(()) => return,
            Err(why) => println!("{}", why)
        }
    }

    async fn ready(&self, ctx: serenity::prelude::Context, ready: Ready) {
        println!("{} is connected!", ready.user.name);

        let test_guild = GuildId(908309641828663297);

        let _reset_command = test_guild
            .create_application_command(&ctx.http, |command| {
                command.name("reset").description("Resets all settings for this Guild to their default values")
            }).await.unwrap();

        let _activate_command = test_guild
            .create_application_command(&ctx.http, |command| {
                command.name("activate").description("Activates the bot for this Guild")
            }).await.unwrap();

        let _deactivate_command = test_guild
            .create_application_command(&ctx.http, |command| {
                command.name("deactivate").description("Deactivates the bot for this Guild")
            }).await.unwrap();

        let _exclude_all_channels_command = test_guild
            .create_application_command(&ctx.http, |command| {
                command
                    .name("exclude_all")
                    .description("Excludes all channels, only channels that you include will be monitored")
            }).await.unwrap();

        let _include_all_channels_command = test_guild
            .create_application_command(&ctx.http, |command| {
                command
                    .name("include_all")
                    .description("Includes all channels, you can still manually exclude channels")
            }).await.unwrap();

        let _exclude_channel_command = GuildId(908309641828663297)
            .create_application_command(&ctx.http, |command| {
                command
                    .name("exclude_channel")
                    .description("Exclude a channel of monitoring")
                    .create_option(|option| {
                        option
                            .name("channel")
                            .description("The channel to exclude")
                            .kind(ApplicationCommandOptionType::Channel)
                            .required(true)
                    })
            }).await.unwrap();

        let _include_channel_command = test_guild
            .create_application_command(&ctx.http, |command| {
                command
                    .name("include_channel")
                    .description("Includes a channel for monitoring")
                    .create_option(|option| {
                        option
                            .name("channel")
                            .description("The channel to include")
                            .kind(ApplicationCommandOptionType::Channel)
                            .required(true)
                    })
            }).await.unwrap();

        // TODO: global, Discord has a 1 hour cache so it's better to use guild specific commands for testing
        // let commands = ApplicationCommand::set_global_application_commands(&ctx.http, |commands| {
        //     commands
        //         .create_application_command(|command| {
        //             command.name("ping").description("A ping command")
        //         })
        // }).await.expect("Unable to create application commands!");
        // println!("I now have the following global slash commands: {:#?}", commands);
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
    let token = std::env::var("DISCORD_TOKEN").expect("Expected a token in the environment!");

    let application_id: u64 = std::env::var("APPLICATION_ID")
        .expect("Expected an application id in the environment!")
        .parse()
        .expect("Unable to parse the provided application id in the environment as a u64!");

    let mut client = Client::builder(&token)
        .event_handler(Handler)
        .application_id(application_id)
        .await.expect("Err creating client");

    {
        let mut data = client.data.write().await;
        data.insert::<GuildSettings>(Arc::new(RwLock::new(HashMap::default())));
    }

    if let Err(why) = client.start().await {
        println!("Client error: {:?}", why);
    }
}
