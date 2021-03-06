use crate::command::{get_args, parse_args, ArgOption, Command, CommandArg, CommandConfig, EMBED_REGULAR_COLOR, EMBED_QUESTION_COLOR};
use serenity::model::channel::{Message, PermissionOverwrite, PermissionOverwriteType};
use serenity::prelude::Context;
use crate::utils::db::{ServerInfo, create_action, ActionType};
use crate::utils::object_finding::get_channel_from_id;
use crate::utils::special_entities_tools::send_to_mod_logs;
use serenity::model::Permissions;
use serenity::model::misc::Mentionable;
use serenity::model::id::RoleId;

pub struct LockCommand;

impl LockCommand {
    fn lock(&self, ctx: &Context, msg: &Message, current_channel: bool, info: &ServerInfo) -> Result<(), String> {
        let channel = if current_channel{
            let g = msg.guild(&ctx.cache).unwrap();
            let g = g.read();
            match g.channels(&ctx.http) {
                Ok(ch) => ch.get(&msg.channel_id).unwrap().clone(),
                Err(_) => return Err("Could not retrieve the channels".to_string())
            }
        } else {
            match get_channel_from_id(ctx, msg, get_args(msg.to_owned(), true), 1)? {
                Some(ch) => ch,
                None => return Ok(())
            }
        };

        let role_id: RoleId = msg.guild_id.unwrap().0.into();
        let mut perm = PermissionOverwrite{
            allow: Permissions::empty(),
            deny: Permissions::SEND_MESSAGES,
            kind: PermissionOverwriteType::Role(msg.guild_id.unwrap().0.into())
        };

        for p in channel.permission_overwrites.iter() {
            if p.kind == PermissionOverwriteType::Role(role_id) {
                perm.allow = p.allow;
                perm.deny = p.deny;
                perm.allow.remove(Permissions::SEND_MESSAGES);
                perm.deny.insert(Permissions::SEND_MESSAGES);
                break
            }
        }

        let action_msg = format!("Channel **{}** has been locked by {}", channel.name, msg.author.name);
        match channel.create_permission(&ctx.http, &perm) {
            Ok(_) => create_action(
                info,
                msg.author.id.to_string(),
                Some(channel.to_string()),
                ActionType::ChannelLock,
                action_msg.to_owned()
            ),
            Err(_) => return Err("Could not lock the channel. Check permissions!".to_string())
        }

        let _ = channel.send_message(ctx.clone().http, |m| {
            m.embed(|e| {
                e.title("Locked!");
                e.description("Channel has been locked!");
                e.color(EMBED_QUESTION_COLOR);
                e
            });
            m
        });

        if !current_channel {
            let _ = msg.channel_id.send_message(ctx.clone().http, |m| {
                m.embed(|e| {
                    e.title("Locked!");
                    e.description(format!("Successfully locked {}!", channel.mention()));
                    e.color(EMBED_REGULAR_COLOR);
                    e
                });
                m
            });
        } else {
            let _ = msg.delete(ctx.clone().http);
        }

        send_to_mod_logs(ctx, info, "Lock", &action_msg);
        Ok(())
    }
}

impl Command for LockCommand {
    fn name(&self) -> String {
        String::from("lock")
    }

    fn desc(&self) -> String {
        String::from("Locks down the channels.")
    }

    fn use_in_dm(&self) -> bool {
        false
    }

    fn args(&self) -> Option<Vec<CommandArg>> {
        Some(vec![
            CommandArg {
                name: "[channel]".to_string(),
                desc: Some("locks down the current channel. If channel will be provided it'll be used instead.".to_string()),
                option: Some(ArgOption::Channel),
                next: None
            }
        ])
    }

    fn perms(&self) -> Option<Vec<String>> {
        Some(vec!["lock".to_string()])
    }

    fn config(&self) -> Option<Vec<CommandConfig>> {
        None
    }

    fn exe(&self, ctx: &Context, msg: &Message, info: &ServerInfo) -> Result<(), String> {
        let args = get_args(msg.clone(), false);
        match parse_args(&self.args().unwrap(), &args) {
            Ok(routes) => {
                self.lock(ctx, msg, routes.is_none(), info)?;
            }
            Err(why) => return Err(why),
        }
        Ok(())
    }
}