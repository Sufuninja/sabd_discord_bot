#[macro_use] extern crate serenity;

use serenity::client::Client;
use serenity::prelude::EventHandler;
use serenity::framework::standard::StandardFramework;
use std::env;

struct Handler;

impl EventHandler for Handler {}

fn main() {
    let mut client = Client::new(&env::var("DISCORD_TOKEN").expect("token"), Handler).expect("Error creating client.");
    client.with_framework(StandardFramework::new().configure(|c| c.prefix(".")).cmd("ping", ping));

    if let Err(why) = client.start() {
        println!("An error occurred while running the client: {:?}", why);
    }
}

command!(ping(_context, message) {
    let _ = message.reply("Pong!");
});