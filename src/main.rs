// This needs high uptime, so .expect, .unwrap inside main loop is not allowed
// But its okay on startup.
extern crate discord;
extern crate serde_yaml;

#[macro_use]
extern crate serde_derive;

use discord::model::ChannelId;
use discord::model::Event;
use discord::Discord;
use discord::Result;

use std::io::prelude::*;
use std::fs::File;
use std::env;

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
struct Config {
    channels: Vec<ChannelConfig>,
    mention_id: String,
    response_channel_id: u64,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
struct SecretConfig {
    token: String,
    is_user_token: bool,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
struct ChannelConfig {
    channel_id: u64,
    name: String,
}

fn load_config(path: &str) -> Config {
    let mut file = File::open(path).expect("couldn't read config file");
    let mut contents = String::new();
    file.read_to_string(&mut contents);
    
    return serde_yaml::from_str(&contents).expect("couldn't parse config file");
}

fn load_secret_config(path: &str) -> SecretConfig {
    let mut file = File::open(path).expect("couldn't read secret config file");
    let mut contents = String::new();
    file.read_to_string(&mut contents);
    
    return serde_yaml::from_str(&contents).expect("couldn't parse secret file");
}

fn connect(secret_config: &SecretConfig) -> Result<Discord> {
    if secret_config.is_user_token {
        println!("Connecting using user token.");
        return Discord::from_user_token(&secret_config.token)
    } else {
        println!("Connecting using bot token.");
        return Discord::from_bot_token(&secret_config.token)
    }
}

fn main() {
    let config_path = env::args().nth(1).expect("Not enough args");
    let secret_config_path = env::args().nth(2).expect("Not enough args");

    let config = load_config(&config_path);
    let secret_config = load_secret_config(&secret_config_path);

    // Loop forever so that we can handle restarts appropriately.
    loop {
        match connect(&secret_config) {
            Ok(discord) => {
                let (mut connection, _)= discord.connect().expect("connect failed");
                println!("Successfully connected.");
                loop {
                    match connection.recv_event() {
                        Ok(Event::MessageCreate(message)) => {
                            for channel_config in &config.channels {
                                if message.channel_id == ChannelId(channel_config.channel_id) {
                                    println!("Got message from channel: {}", channel_config.name);
                                    let sending_channel_id = ChannelId(config.response_channel_id);
                                    let _ = discord.send_message(sending_channel_id, &format!("<@&{}> Recruitment message in {} from {}", config.mention_id, channel_config.name, message.author.name), "", false);
                                    let _ = discord.send_message(sending_channel_id, &message.content, "", false);
                                }
                            }
                        }
                        Ok(_) => {}
                        Err(discord::Error::Closed(code, _body)) => {
                            println!("Gateway closed, error code {:?}", code);
                            break
                        }
                        Err(err) => {
                            println!("Error received: {:?}", err);
                        }
                    }
                }
            }
            Err(err) => {
                println!("Error connecting: {:?}", err);
            }
        }
    }
}
