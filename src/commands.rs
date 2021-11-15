use serenity::builder::CreateApplicationCommands;
use serenity::model::interactions::application_command::ApplicationCommandOptionType;

pub fn create_commands(commands: &mut CreateApplicationCommands) -> &mut CreateApplicationCommands {
    // TODO: permissions
    // permissions for commands are really fiddly, the commands for the bot are all admin-only
    // and they recently changed the behavior of default_permission: https://github.com/discord/discord-api-docs/commit/1e66727c499868d9fcdd471efbb71fca48d396ad
    // before: only admins can use the command
    // now: nobody can use the command

    commands
        .create_application_command(|command| {
            command
                .name("reset")
                .description("Resets all settings for this Guild to their default values")
                //.default_permission(false)
        })
        .create_application_command(|command| {
            command
                .name("activate")
                .description("Activates the bot for this Guild")
                //.default_permission(false)
        })
        .create_application_command( |command| {
            command
                .name("deactivate")
                .description("Deactivates the bot for this Guild")
                //.default_permission(false)
        })
        .create_application_command(|command| {
            command
                .name("exclude_all")
                .description("Excludes all channels, only channels that you include will be monitored")
                //.default_permission(false)
        })
        .create_application_command(|command| {
            command
                .name("include_all")
                .description("Includes all channels, you can still manually exclude channels")
                //.default_permission(false)
        })
        .create_application_command(|command| {
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
                //.default_permission(false)
        })
        .create_application_command(|command| {
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
                //.default_permission(false)
        })
        .create_application_command(|command| {
            command
                .name("set_log_channel")
                .description("Sets the log channel containing the reports")
                .create_option(|option| {
                    option
                        .name("channel")
                        .description("The log channel")
                        .kind(ApplicationCommandOptionType::Channel)
                        .required(true)
                })
                //.default_permission(false)
        })
}
