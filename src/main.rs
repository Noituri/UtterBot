#[macro_use]
extern crate diesel;

#[cfg(test)]
mod tests;
mod config;
mod handler;
mod bot_modules;
mod command;
mod database;
mod utils;

use log::{info, error};
use serenity::Client;
use handler::*;
use bot_modules::*;

fn main() {
    pretty_env_logger::init();
    info!("Initializing database...");
    { let _ = database::get_db_con(); }

    info!("Starting bot...");
    let mut client = Client::new(&config::BOT_CONFIG.token, Handler).expect("Err creating client");
    if let Err(why) = client.start() {
        error!("Client error: {:?}", why);
    }
}
