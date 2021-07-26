use std::borrow::Cow;

use pilot::{bot::Bot, typing::UpdateMessage, method::send::SendMessage, method::DeleteMessage};
#[tokio::main]
async fn main() {
    let mut bot = Bot::new();
    bot.command("ping", |bot, msg| async move {
        match msg.as_ref() {
            UpdateMessage::Message(msg) => {
                let message = SendMessage::new(msg.chat.id.to_string(), "pong");
                bot.request(message).await.unwrap();
            }
            _ => {}
        }
    });
    bot.other(|bot, msg| async move {
        match msg.as_ref() {
            UpdateMessage::Message(new_msg) => {
                let chat_id = new_msg.chat.as_ref().id.to_string();
                if let Some(text) = &new_msg.text {
                    let date = new_msg.date;
                   let username= &new_msg.from.as_ref().unwrap().username.as_ref().unwrap();
                    println!("[{}][{}] {}: {}", chat_id, date, username, text);
                    let delete_message = DeleteMessage{
                        chat_id: Cow::Borrowed(chat_id.as_ref()),
                        message_id: new_msg.message_id,
                    };
                    bot.request(delete_message).await;
                }
            },
            _ => {}
        }
         
    });
    bot.polling().await;
}