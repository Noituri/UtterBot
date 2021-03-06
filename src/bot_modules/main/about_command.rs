use crate::command::{Command, CommandArg, CommandConfig, EMBED_REGULAR_COLOR};
use crate::config::VERSION;
use serenity::model::channel::Message;
use serenity::prelude::Context;
use crate::utils::db::ServerInfo;

pub struct AboutCommand;

impl Command for AboutCommand {
    fn name(&self) -> String {
        String::from("about")
    }

    fn desc(&self) -> String {
        String::from("Shows information about this bot.")
    }

    fn use_in_dm(&self) -> bool {
        true
    }

    fn args(&self) -> Option<Vec<CommandArg>> {
        None
    }

    fn perms(&self) -> Option<Vec<String>> {
        None
    }

    fn config(&self) -> Option<Vec<CommandConfig>> {
        None
    }

    fn exe(&self, ctx: &Context, msg: &Message, _: &ServerInfo) -> Result<(), String> {
        let _ = msg.channel_id.send_message(&ctx.http, |m| {
            m.embed(|e| {
                e.title("About");
                e.description(format!(
                    "**Created by:**\n\
                     Mikołaj '[Noituri](https://github.com/noituri)' Radkowski\n\n\
                     **Source code:**\n\
                     Link -> [click](https://github.com/noituri/utterbot)\n\
                     Discord Library -> [serenity](https://github.com/serenity-rs/serenity)\n\n\
                     **Version:**\n\
                     {}",
                    VERSION
                ));
                e.color(EMBED_REGULAR_COLOR);
                e
            });
            m
        });

        Ok(())
    }
}
