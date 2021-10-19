#[macro_use]
extern crate log;

use std::borrow::Cow;

use crate::github::upload_to_github;
use once_cell::sync::{Lazy, OnceCell};
use pilot::method::SendMediaGroup;
use pilot::{bot::Bot, method::send::SendMessage, method::DeleteMessage, typing::UpdateMessage};
use std::sync::Arc;
use tokio::sync::mpsc;
use tokio::sync::mpsc::{Receiver, Sender};
use tokio::sync::mpsc::error::SendError;
use tokio::time::Duration;

pub mod github;

#[derive(Debug)]
pub struct Opts {
    telegram_token: String,
    chat: String,
    github_token: String,
    repo: String,
}

static CHANNEL: OnceCell<Sender<(Arc<Bot>, Arc<UpdateMessage>)>> = OnceCell::new();

#[tokio::main]
async fn main() {
    env_logger::init();

    let opt = Arc::new(Opts {
        chat: std::env::var("CHAT").expect("CHAT must be set"),
        telegram_token: std::env::var("TELEGRAM_TOKEN").expect("TELEGRAM_TOKEN must be set"),
        github_token: std::env::var("GITHUB_TOKEN").expect("GITHUB_TOKEN must be set"),
        repo: std::env::var("REPO").expect("REPO must be set"),
    });
    debug!("chat is {}", opt.chat);
    debug!("telegram token is {}", opt.telegram_token);
    debug!("github token is {}", opt.github_token);
    debug!("github repo is {}", opt.repo);
    std::env::set_var("TELEGRAM_BOT_SECRET_KEY", &opt.telegram_token);

    let (sender, mut receiver) = tokio::sync::mpsc::channel(100);

    CHANNEL.set(sender).expect("cannot set global channel");
    let mut bot = Bot::new();

    bot.other(|bot, msg| async move {
        debug!("bot get msg");
        match CHANNEL.get().unwrap().send((bot, msg)).await {
            Ok(_) => {}
            Err(e) => {
                error!("cannot send msg to channel: {}", e);
            }
        };
    });

    let consumer = tokio::spawn(async move {
        while let Some((bot, msg)) = receiver.recv().await {
            match msg.as_ref() {
                UpdateMessage::Message(new_msg) => {
                    let chat_id = new_msg.chat.as_ref().id.to_string();

                    let chat_whitelist = std::env::var("CHAT").unwrap();
                    if !chat_id.eq(&chat_whitelist) {
                        warn!("sender[{}] does not equal to whitelist", &chat_id);
                        let reply = SendMessage::new(&chat_id, "your are not in whitelist");
                        bot.request(reply).await;
                        return;
                    }

                    if let Some(text) = &new_msg.text {
                        let msg_id = new_msg.message_id;

                        loop {
                            let result = upload_to_github(opt.clone(), text.clone()).await;
                            match result {
                                Ok(_) => {
                                    loop {
                                        info!("deleting telegram msg id={}", &chat_id);
                                        let delete_message = DeleteMessage {
                                            chat_id: Cow::Owned(chat_id.clone()),
                                            message_id: msg_id,
                                        };
                                        match bot.request(delete_message).await {
                                            Ok(_) => break,
                                            Err(e) => {
                                                warn!("bot request fail: {}", e);
                                            }
                                        }
                                    }
                                    break;
                                }
                                Err(e) => {
                                    warn!("update to github got error: {}", e);
                                }
                            }
                        }
                    }
                }
                _ => {}
            }
        }
    });
    info!("bot is listening");
    bot.polling().await;
    info!("bot is stopping");
}
