#[macro_use]
extern crate log;

use std::borrow::Cow;
use std::sync::Arc;

use once_cell::sync::{Lazy, OnceCell};
use teloxide::prelude::*;
use teloxide::RequestError;
use tokio::sync::mpsc;
use tokio::sync::mpsc::{Receiver, Sender};
use tokio::sync::mpsc::error::SendError;
use tokio::time::Duration;

use crate::github::upload_to_github;

pub mod github;

#[derive(Debug)]
pub struct Opts {
    telegram_token: String,
    chat: String,
    github_token: String,
    repo: String,
}


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

    let bot = Bot::new(&opt.telegram_token);

    info!("bot is listening");
    teloxide::repl(bot, |bot: Bot, msg: Message| async move {
       let opt = Arc::new(Opts {
            chat: std::env::var("CHAT").expect("CHAT must be set"),
            telegram_token: std::env::var("TELEGRAM_TOKEN").expect("TELEGRAM_TOKEN must be set"),
            github_token: std::env::var("GITHUB_TOKEN").expect("GITHUB_TOKEN must be set"),
            repo: std::env::var("REPO").expect("REPO must be set"),
        });
        let chat_id = msg.chat.id;
        let chat_whitelist = std::env::var("CHAT").unwrap();

        if !chat_id.to_string().eq(&chat_whitelist) {
            warn!("sender[{}] does not equal to whitelist", &chat_id);
            bot.send_message(chat_id, "your are not in whitelist").await?;
            return Ok(());
        }

        if let Some(text) = msg.text() {
            let msg_id = msg.id;
            let msg_date = msg.date.timestamp() as i32;
            loop {
                let result = upload_to_github(opt.clone(), text.to_string(), msg_date).await;
                match result {
                    Ok(_) => {
                        loop {
                            info!("deleting telegram msg id={}", &msg_id);
                            match bot.delete_message(chat_id, msg_id).await {
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
        Ok(())
    }).await;

    info!("bot is stopping");
}
