use crate::command::EMBED_QUESTION_COLOR;
use serenity::prelude::Context;
use serenity::model::channel::{Message, GuildChannel, ChannelType};
use serenity::model::guild;
use chrono::{Utc, DateTime};
use crate::handler::STATE;
use serenity::model::id::{GuildId, UserId};
use std::str::FromStr;
use serenity::model::guild::Member;

#[allow(dead_code)]
#[derive(Clone)]
pub enum FindType {
    Role,
    User,
    Channel,
}

#[derive(Clone)]
pub struct FindsAwaitingAnswer {
    pub who: u64,
    pub channel: u64,
    pub when: DateTime<Utc>,
    pub finds: Vec<(u64, String)>,
    pub args: Vec<String>,
    pub replace_index: usize,
}

pub trait FindObject {
    fn get_id(&self) -> u64;
    fn get_name(&self) -> String;
}
impl FindObject for GuildChannel {
    fn get_id(&self) -> u64 {
        self.id.0
    }
    fn get_name(&self) -> String {
        match self.kind {
            ChannelType::Text => format!("{} {}", "🗒️", self.name.to_owned()),
            ChannelType::Voice => format!("{} {}", "🎙️", self.name.to_owned()),
            _ => format!("{} {}", "📂", self.name.to_owned())
        }
    }
}
impl FindObject for guild::Role {
    fn get_id(&self) -> u64 {
        self.id.0
    }
    fn get_name(&self) -> String {
        self.name.to_owned()
    }
}
impl FindObject for guild::Member {
    fn get_id(&self) -> u64 {
        self.user_id().0
    }
    fn get_name(&self) -> String {
        self.user.read().name.to_owned()
    }
}

impl FindObject for &guild::Member {
    fn get_id(&self) -> u64 {
        self.user_id().0
    }
    fn get_name(&self) -> String {
        self.user.read().name.to_owned()
    }
}

pub fn find_object<T>(ctx: &Context, msg: &Message, objects: Vec<T>, args: &Vec<String>, a_index: usize, obj_type: FindType) -> Result<u64, String>
    where T: FindObject {
    let find_text = &args[a_index];
    let obj_name = match obj_type {
        FindType::Channel => "channel",
        FindType::Role => "role",
        FindType::User => "user"
    };

    let mut matched_objects: Vec<(u64, String)> = Vec::new();
    for v in objects.iter() {
        if v.get_name().to_lowercase().contains(&find_text.to_lowercase()) {
            matched_objects.push((v.get_id(), format!("**{}.** {}\n", matched_objects.len()+1, v.get_name())))
        }
    }

    match matched_objects.len() {
        0 => return Err(format!("Could not find requested {}!", obj_name)),
        1 => return Ok(matched_objects[0].0),
        l if l > 15 => return Err("Too many results. Please be more specific.".to_string()),
        _ => {
            let mut description = String::new();
            matched_objects.iter().for_each(|r| description.push_str(&r.1));
            {
                let mut state = STATE.lock().unwrap();
                let tmp_find = FindsAwaitingAnswer{
                    who: msg.author.id.0,
                    channel: msg.channel_id.0,
                    when: Utc::now(),
                    finds: matched_objects,
                    args: args.clone(),
                    replace_index: a_index
                };

                let mut exists = false;
                for (i, v) in state.role_finds_awaiting.iter().enumerate() {
                    if v.who == msg.author.id.0 {
                        exists = true;
                        state.role_finds_awaiting[i] = tmp_find.clone();
                        break
                    }
                }

                if !exists {
                    state.role_finds_awaiting.push(tmp_find);
                }
            }

            let _ = msg.channel_id.send_message(&ctx.http, |m| {
                m.embed(|e| {
                    e.title(format!("Which {} did you have in mind?", obj_name));
                    e.color(EMBED_QUESTION_COLOR);
                    e.description(description);
                    e.footer(|f| {
                        f.text(format!("Respond with number corresponding to the {}.", obj_name));
                        f
                    });
                    e
                });
                m
            });
        }
    }
    Ok(0)
}

pub fn get_role_from_id(ctx: &Context, msg: &Message, mut args: Vec<String>, a_index: usize) -> Result<Option<guild::Role>, String> {
    let mut tmp_id = args[a_index].to_owned();
    if msg.mention_roles.len() != 0 {
        tmp_id = msg.mention_roles[0].to_string();
    }
    let g_roles = if let Ok(guild_roles) = ctx.http.get_guild_roles(msg.guild_id.unwrap().0) {
        guild_roles
    } else {
        return Err("Could not retrieve guild roles!".to_string())
    };

    for v in g_roles.iter() {
        if &v.id.to_string() == &tmp_id{
            return Ok(Some(v.clone()))
        }
    }

    let found_role = find_object(ctx, msg, g_roles, &args, a_index,FindType::Role)?;
    if found_role != 0 {
        args[a_index] = found_role.to_string();
        return get_role_from_id(ctx, msg, args, a_index)
    }
    Ok(None)
}

pub fn get_channel_from_id(ctx: &Context, msg: &Message, mut args: Vec<String>, a_index: usize) -> Result<Option<GuildChannel>, String> {
    let mut tmp_id = args[a_index].to_owned();
    match &msg.mention_channels {
        Some(mch) => if mch.len() != 0 {
            tmp_id = mch[0].id.to_string();
        }
        None => {}
    }

    let channels = match ctx.http.get_channels(msg.guild_id.unwrap().0) {
        Ok(ch) => ch,
        Err(_) => return Err("Could not retrieve guild channels!".to_string())
    };

    for v in channels.iter() {
        if v.kind != ChannelType::Text {
            continue
        }
        if &v.id.to_string() == &tmp_id{
            return Ok(Some(v.clone()))
        }
    }

    let found_channel = find_object(ctx, msg, channels, &args, a_index, FindType::Channel)?;
    if found_channel != 0 {
        args[a_index] = found_channel.to_string();
        return get_channel_from_id(ctx, msg, args, a_index)
    }
    Ok(None)
}

pub fn get_member_from_id(ctx: &Context, msg: &Message, mut args: Vec<String>, a_index: usize) -> Result<Option<guild::Member>, String> {
    let tmp_id =  if msg.mentions.len() > 0 {
        msg.mentions[0].id.to_string()
    } else {
        args[a_index].to_owned()
    };


    let members: Vec<Member> = match msg.guild(&ctx.cache) {
        Some(g) => {
            let guild = g.read();
            if let Ok(u) = UserId::from_str(&tmp_id) {
                if let Some(m) = guild.members.get(&u) {
                    return Ok(Some(m.clone()))
                }
            }

            guild.members_username_containing(&tmp_id, false, false)
                .iter()
                .map(|m| m.clone())
                .cloned()
                .collect()
        },
        None => {
            get_guild_members(ctx, msg.guild_id.unwrap())?
        },
    };

    for v in members.iter() {
        if v.user_id().to_string() == tmp_id {
            return Ok(Some(v.clone().clone()))
        }
    }

    let found_user = find_object(ctx, msg, members, &args, a_index, FindType::User)?;
    if found_user != 0 {
        args[a_index] = found_user.to_string();
        return get_member_from_id(ctx, msg, args, a_index)
    }
    Ok(None)
}

pub fn get_guild_members(ctx: &Context, guild_id: GuildId) -> Result<Vec<guild::Member>, String> {
    let limit = match &ctx.cache.read().guilds.get(&guild_id) {
        Some(g) => {
            let member_count = g.read().member_count;
            if member_count > 1000 {
                1000
            } else {
                member_count
            }
        },
        None => 1000
    };

    match ctx.http.get_guild_members(guild_id.0, Some(limit), None) {
        Ok(m) => Ok(m),
        Err(_) => Err("Could not retrieve guild members!".to_string())
    }
}