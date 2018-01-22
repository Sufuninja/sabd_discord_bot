#[macro_use] 
extern crate serenity;
extern crate typemap;
// Serenity library imports
use serenity::client::bridge::gateway::{ShardId, ShardManager};
use serenity::framework::standard::{Args, DispatchError, StandardFramework, HelpBehaviour, CommandOptions, help_commands};
use serenity::model::channel::Message;
use serenity::model::gateway::Ready;
use serenity::model::Permissions;
use serenity::prelude::Mutex;
use serenity::prelude::*;
// Standard library imports
use std::collections::HashMap;
use std::env;
use std::fmt::Write;
use std::sync::Arc;
// Typemap imports
use typemap::Key;

struct ShardManagerContainer;

impl Key for ShardManagerContainer {
    type Value = Arc<Mutex<ShardManager>>;
}

struct CommandCounter;

impl Key for CommandCounter {
    type Value = HashMap<String, u64>;
}

struct Handler;

impl EventHandler for Handler {
    fn ready(&self, _: Context, ready: Ready) {
        println!("{} is connected!", ready.user.name);
    }
}

fn main() {
    let token = env::var("DISCORD_TOKEN").expect("Expected a token in the environment",);
    let mut client = Client::new(&token, Handler).expect("Error creating client.");
    {
        let mut data = client.data.lock();
        data.insert::<CommandCounter>(HashMap::default());
        data.insert::<ShardManagerContainer>(Arc::clone(&client.shard_manager));
    }

    client.with_framework(
        // Configures the client, allowing for options to mutate how the framework functions.
        // Refer to the docs for `serenity::ext::framework::Configuration` for all available
        // configurations.
        StandardFramework::new()
        .configure(|c| c
            .allow_whitespace(true)
            .on_mention(true)
            .prefix(".")
            .delimiters(vec![", ", ","])
        )
        .before(|ctx, msg, command_name| {
            println!("Got command '{}' by user '{}'",
                    command_name,
                    msg.author.name);
            
            let mut data = ctx.data.lock();
            let counter = data.get_mut::<CommandCounter>().unwrap();
            let entry = counter.entry(command_name.to_string()).or_insert(0);
            *entry += 1;

            true // if `before` returns false, command processing doesn't happen.
        })
        // Similar to `before`, except will be called directly after command exec
        .after(|_, _, command_name, error| {
            match error {
                Ok(()) => println!("Processed command '{}'", command_name),
                Err(why) => println!("Command '{}' return error {:?}", command_name, why),
            }
        })
        // Set a function that is called whenever a command's execution didn't complete for one
        // reason or another. For example, when a user has exceeded a rate-limit or a command
        // can only be performed by the bot owner.
        .on_dispatch_error(|_ctx, msg, error| {
            if let DispatchError::RateLimited(seconds) = error {
                let _ = msg.channel_id.say(&format!("Try this again in {} seconds.", seconds));
            }
        })
        // Can't be used more than once per five seconds
        .simple_bucket("emoji", 5)
        // Can't be used more than 2 times per 30 seconds, with a 5 second delay
        .bucket("complicated", 5, 30, 2)
        .command("about", |c| c.cmd(about))
        // You can use the simple `help(help_commands::with_embeds)` or customise your help-menu via `customised_help()`.
        .customised_help(help_commands::with_embeds, |c| {
            // This replaces the information that a user can pass a command name as argument to gain specific information
            // about it.
            c.individual_command_tip("Hello!\n\
            If you want more information about a specific command, just pass the command as an argument.")
            // Some commands require a `{}` to replace it by the actual name.
            // In this case, it's the command's name.
            .command_not_found_text("Cound not find {} as a valid command. Sorry! :frowning:")
            // This is the second command requiring `{}` to replace the actual name.
            .suggestion_text("How about this command: {}? It's so hot right now.")
            // You can set help menu filter behavior
            // Here are all possible cases
            // If a user lacks permissions for a command, we can hide it
            .lacking_permissions(HelpBehaviour::Hide)
            // If user is nothing but lacking a role, display nothing
            .lacking_role(HelpBehaviour::Nothing)
            // `Strike` the command
            .wrong_channel(HelpBehaviour::Strike)
        })
        .command("commands", |c| c
            // Make this command use the "complicated" bucket
            .bucket("complicated")
            .cmd(commands))
        .group("Emoji", |g| g
            .prefix("emoji")
            .command("cat", |c| c
                .desc("Sends an emoji with a cat.")
                .batch_known_as(vec!["kitty", "neko"])
                .bucket("emoji")
                .cmd(cat)
                .required_permissions(Permissions::ADMINISTRATOR)
            )
            .command("dog", |c| c
                .desc("Sends an emoji with a dog.")
                .bucket("emoji")
                .cmd(dog)
            )
        )
        .command("multiply", |c| c
            .known_as("*") //Lets us call .* instead of .multiply
            .cmd(multiply)
        )
        .command("latency", |c| c
            .cmd(latency)
        )
        .command("ping", |c| c
            .check(owner_check)
            .cmd(ping)
        )
        .command("role", |c| c
            .allowed_roles(vec!["organizers", "altcoin god"])
        )
        .command("some long command", |c| c.cmd(some_long_command)),
    );
}

// Commands are created using the command! macro
command!(commands(ctx, msg, _args) {
    let mut contents = "Commands used:\n".to_string();

    let data = ctx.data.lock();
    let counter = data.get::<CommandCounter>().unwrap();

    for (k, v) in counter {
        let _ = write!(contents, "- {name}: {amount}\n", name=k, amount=v);
    }

    if let Err(why) = msg.channel_id.say(&contents) {
        println!("Error sending message: {:?}", why);
    }
});

// A function which acts as a check to determine whether to call a command
fn owner_check(_: &mut Context, msg: &Message, _: &mut Args, _: &CommandOptions) -> bool {
    msg.author.id == 6712
}

command!(some_long_command(_ctx, msg, args) {
    if let Err(why) = msg.channel_id.say(&format!("Arguments {:?}", args)) {
        println!("Error sending message: {:?}", why);
    }
});

command!(about_role(_ctx, msg, args) {
    let potential_role_name = args.full();

    if let Some(guild) = msg.guild() {
        // `role_by_name()` allows us to attempt attaining a reference to a role via its name
        if let Some(role) = guild.read().role_by_name(&potential_role_name) {
            if let Err(why) = msg.channel_id.say(&format!("Role-ID: {}", role.id)) {
                println!("Error sending message: {:?}", why);
            }

            return Ok(());
        }
    }

    if let Err(why) = msg.channel_id.say(&format!("Could not find role named: {:?}", potential_role_name)) {
        println!("Error sending message: {:?}", why);
    }
});

command!(multiply(_ctx, msg, args) {
    let first = args.single::<f64>().unwrap();
    let second = args.single::<f64>().unwrap();

    let res = first * second;

    if let Err(why) = msg.channel_id.say(&res.to_string()) {
        println!("Err sending product of {} and {}: {:?}", first, second, why);
    }
});

command!(about(_ctx, msg, _args) {
    if let Err(why) = msg.channel_id.say("This is a small test-bot! : )") {
        println!("Error sending message: {:?}", why);
    }
});

command!(latency(ctx, msg, _args) {
    // The shard manager is an interface for mutating, stopping, restarting, and
    // retrieving information about shards.
    let data = ctx.data.lock();

    let shard_manager = match data.get::<ShardManagerContainer>() {
        Some(v) => v,
        None => {
            let _ = msg.reply("There was a problem getting the shard manager");

            return Ok(());
        },
    };

    let manager = shard_manager.lock();
    let runners = manager.runners.lock();

    // Shards are backed by a "shard runner" responsible for processing events
    // over the shard, so we'll get the information about the shard runner for
    // the shard this command was sent over.
    let runner = match runners.get(&ShardId(ctx.shard_id)) {
        Some(runner) => runner,
        None => {
            let _ = msg.reply("No shard found");

            return Ok(());
        },
    };

    let _ = msg.reply(&format!("The shard latency is {:?}", runner.latency));
});

command!(ping(_ctx, msg, _args) {
    if let Err(why) = msg.channel_id.say("Pong! :panda_face:") {
        println!("Error sending message: {:?}", why);
    }
});

command!(dog(_ctx, msg, _args) {
    if let Err(why) = msg.channel_id.say(":dog:") {
        println!("Error sending message: {:?}", why);
    }
});

command!(cat(_ctx, msg, _args) {
    if let Err(why) = msg.channel_id.say(":cat:") {
        println!("Error sending message: {:?}", why);
    }
});