use std::env;
use caseus_foratus_helveticus::{DinnerService, Service, ReminderService, ClockTime, GreeterService};
use teloxide::prelude::*;
use tokio::task::JoinSet;

#[tokio::main]
async fn main() {
    pretty_env_logger::init();

    let args: Vec<String> = env::args().collect();
    let chat_id: i64 = args[1].parse().unwrap();

    log::info!("Starting caseus bot with chat id {}...", chat_id);

    let bot = Bot::from_env();

    let mut tasks = JoinSet::new();

    // Greeter
    let greeter = GreeterService::new(bot.clone(), ChatId(chat_id).into());
    tasks.spawn(async move {
        greeter.run().await;
    });

    // Dinner
    let dinner_service = DinnerService::new(bot.clone(), ChatId(chat_id).into());
    tasks.spawn(async move {
        dinner_service.run().await;
    });

    // Trash reminder
    let trash_reminder = ReminderService::new(
        bot.clone(),
        ChatId(chat_id).into(),
        "♻️🗑️ <b>Tomorrow is trash collection day!</b>\n<b>Don't forget to take it out!</b> 👀".to_string(),
        "dates/trash-dates.txt".to_string(),
        ClockTime::new(20, 0, 0)
    );
    tasks.spawn(async move {
        trash_reminder.run().await;
    });

    // Compost reminder
    let compost_reminder = ReminderService::new(
        bot.clone(),
        ChatId(chat_id).into(),
        "♻️🍂️ <b>Tomorrow is compost collection day!</b>\n<b>Don't forget to take it out!</b> 👀".to_string(),
        "dates/compost-dates.txt".to_string(),
        ClockTime::new(20, 0, 0)
    );
    tasks.spawn(async move {
        compost_reminder.run().await;
    });

    // Glass reminder
    let glass_reminder = ReminderService::new(
        bot.clone(),
        ChatId(chat_id).into(),
        "♻️🫙 <b>Tomorrow is glass collection day!</b>\n<b>Don't forget to take it out!</b> 👀".to_string(),
        "dates/glass-dates.txt".to_string(),
        ClockTime::new(20, 0, 0)
    );
    tasks.spawn(async move {
        glass_reminder.run().await;
    });

    // Paper reminder
    let paper_reminder = ReminderService::new(
        bot.clone(),
        ChatId(chat_id).into(),
        "♻️🗞️ <b>Tomorrow is paper collection day!</b>\n<b>Don't forget to take it out!</b> 👀".to_string(),
        "dates/paper-dates.txt".to_string(),
        ClockTime::new(20, 0, 0)
    );
    tasks.spawn(async move {
        paper_reminder.run().await;
    });

    // Wait forever
    while let Some(_) = tasks.join_next().await {}
}
