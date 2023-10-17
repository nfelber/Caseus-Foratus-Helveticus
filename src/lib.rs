use async_trait::async_trait;
use chrono::{Duration, Utc, TimeZone, FixedOffset, Timelike, Weekday, DateTime, Datelike};
use rand::Rng;
use serde_json::Value;
use teloxide::{prelude::*, types::{Recipient, MessageId, InputFile}, RequestError};
use tokio::{time::sleep, fs::File, io::{BufReader, AsyncBufReadExt, AsyncWriteExt}};
use url::Url;

pub struct ClockTime {
    hour: u32,
    minute: u32,
    second: u32
}

impl ClockTime {
    pub fn new(hour: u32, minute: u32, second: u32) -> Self {
        assert!(hour < 24);
        assert!(minute < 60);
        assert!(second < 60);
        Self { hour, minute, second }
    }
}

fn utc_2_00_now() -> DateTime<FixedOffset> {
    FixedOffset::east_opt(2 * 3600).unwrap().from_utc_datetime(&Utc::now().naive_utc())
}

async fn sleep_until_clock_time(clock_time: &ClockTime) {
    let now = utc_2_00_now();
    let mut next_time = now.with_hour(clock_time.hour).unwrap()
        .with_minute(clock_time.minute).unwrap()
        .with_second(clock_time.second).unwrap()
        .with_nanosecond(0).unwrap();
    while now > next_time { next_time += Duration::days(1); }

    sleep(next_time.signed_duration_since(now).to_std().unwrap()).await;
}


#[async_trait]
pub trait Service {
    async fn run(&self);
}

pub struct GreeterService {
    bot: Bot,
    chat_id: Recipient,
}

impl GreeterService {
    pub fn new(bot: Bot, chat_id: Recipient) -> Self {
        Self { bot, chat_id }
    }

    async fn send_greeting(&self) -> Result<(), RequestError>  {
        let response = reqwest::get("https://api.nasa.gov/planetary/apod?api_key=DEMO_KEY").await?;
        let body = response.text().await.unwrap();
        let v: Value = serde_json::from_str(&body).unwrap();
        let url = v["hdurl"].as_str().unwrap();
        self.bot.send_photo(self.chat_id.clone(), InputFile::url(Url::parse(url).unwrap())).await?;
        self.bot.send_message(self.chat_id.clone(), format!("\
                ☀️ <b>Good morning everyone!</b>\n\
                🪐 <b>Let's wake up to today's <a href=\"https://apod.nasa.gov/apod/astropix.html\">APOD</a>: <i>{}</i></b>\n\
                <i>{}</i>", v["title"], v["explanation"]))
            .parse_mode(teloxide::types::ParseMode::Html).await?;
        Ok(())
    }
}

#[async_trait]
impl Service for GreeterService {
    async fn run(&self) {
        loop {
            // Sleep until the morning
            sleep_until_clock_time(&ClockTime::new(7, 0, 0)).await;
            self.send_greeting().await.unwrap_or_else(|e| log::error!("Couldn't send greeting ({}).", e));
        }
    }
}

pub struct DinnerService {
    bot: Bot,
    chat_id: Recipient,
}

impl DinnerService {
    pub fn new(bot: Bot, chat_id: Recipient) -> Self {
        Self { bot, chat_id }
    }

    async fn send_attendance_poll(&self) -> Result<MessageId, RequestError>  {
        let emojis = ['🍔', '🍕', '🌮', '🌯', '🥙', '🥘', '🍝', '🫕', '🥗', '🍲', '🍛', '🍜'];
        let emoji = emojis[rand::thread_rng().gen_range(0..emojis.len())];
        let m = self.bot.send_poll(
            self.chat_id.clone(),
            format!("Who's joining for dinner tonight? {}\n\
                Let our cook know how many people to expect! 👩‍🍳", emoji),
            [
                "I'll be there! 🍽".into(),
                "I'll bring a friend! 🍽🍽".into(),
                "Not tonight...".into()
                ]
            ).is_anonymous(false).await?;
        Ok(m.id)
    }

    async fn send_schedule_poll(&self) -> Result<MessageId, RequestError>  {
        let m = self.bot.send_poll(
            self.chat_id.clone(),
            "🗓 Let's define this week's dinner schedule!\n\
                Pick the day which fits you the best.",
            [
                "I'll cook on Monday! 🌻".into(),
                "I'll cook on Tuesday! 🦦".into(),
                "I'll cook on Wednsday! 🌈".into(),
                "I'll cook on Thursday! 🐙".into(),
                "I'd rather not cook this week. 👀".into()
                ]
            ).is_anonymous(false).await?;
        Ok(m.id)
    }

    async fn stop_poll(&self, id: MessageId) -> Result<(), RequestError> {
        self.bot.stop_poll(
            self.chat_id.clone(),
            id
            ).await?;
        Ok(())
    }
}

#[async_trait]
impl Service for DinnerService {
    async fn run(&self) {
        loop {
            // Sleep until the morning
            sleep_until_clock_time(&ClockTime::new(8, 0, 0)).await;

            let weekday = utc_2_00_now().weekday();
            if weekday.num_days_from_monday() <= 3 {
                // From Monday to Thursday
                let id = match self.send_attendance_poll().await {
                    Ok(id) => id,
                    Err(e) => {
                        log::error!("Couldn't send attendance poll ({}).", e);
                        continue;
                    }
                };

                // Sleep until the afternoon
                sleep_until_clock_time(&ClockTime::new(16, 0, 0)).await;
                self.stop_poll(id).await.unwrap_or_else(|e| log::error!("Couldn't stop attendance poll ({}).", e));
            } else if weekday == Weekday::Sun {
                // On Sunday
                let id = match self.send_schedule_poll().await {
                    Ok(id) => id,
                    Err(e) => {
                        log::error!("Couldn't send schedule poll ({}).", e);
                        continue;
                    }
                };

                // Sleep until midnight
                sleep_until_clock_time(&ClockTime::new(0, 0, 0)).await;
                self.stop_poll(id).await.unwrap_or_else(|e| log::error!("Couldn't stop schedule poll ({}).", e));
            }
        }
    }
}

pub struct ReminderService {
    bot: Bot,
    chat_id: Recipient,
    message: String,
    dates_filename: String,
    clock_time: ClockTime
}

impl ReminderService {
    pub fn new(bot: Bot, chat_id: Recipient, message: String, dates_filename: String, clock_time: ClockTime) -> Self {
        Self { bot, chat_id, message, dates_filename, clock_time }
    }

    async fn send_message(&self) -> Result<(), RequestError> {
        self.bot.send_message(
            self.chat_id.clone(),
            &self.message
        ).parse_mode(teloxide::types::ParseMode::Html).await?;
        Ok(())
    }
}

#[async_trait]
impl Service for ReminderService {
    async fn run(&self) {
        loop {
            // Sleep until next reminder clock time
            sleep_until_clock_time(&self.clock_time).await;

            let file = match File::open(&self.dates_filename).await {
                Ok(f) => f,
                Err(e) => {
                    log::error!("Couldn't open file {} ({}).", self.dates_filename, e);
                    continue;
                },
            };
            let reader = BufReader::new(file);

            let mut lines = reader.lines();
            let mut lines_vec: Vec<String> = vec![];
            while let Some(line) = lines.next_line().await.unwrap() {
                lines_vec.push(line);
            }

            let mut file = match File::create(&self.dates_filename).await {
                Ok(f) => f,
                Err(e) => {
                    log::error!("Couldn't create file {} ({}).", self.dates_filename, e);
                    continue;
                },
            };


            for line in lines_vec {
                let now = utc_2_00_now();
                let [day, month, year]: [u32; 3] = match line.split('.').map(|s| s.parse()).collect::<Result<Vec<u32>,_>>() {
                    Ok(v) => match v.try_into() {
                        Ok(a) => a,
                        Err(_) => {
                            log::error!("Couldn't parse date {} - skipping.", line);
                            continue;
                        },
                    },
                    Err(e) => {
                        log::error!("Couldn't parse date {} ({}) - skipping.", line, e);
                        continue;
                    },
                };
                let date = now.with_day(day).unwrap().with_month(month).unwrap().with_year(year as i32).unwrap();

                if now == date {
                    self.send_message().await.unwrap_or_else(|e| log::error!("Couldn't send message ({}).", e));
                } else if now < date {
                    file.write_all(line.as_bytes()).await.unwrap();
                    file.write_all(b"\n").await.unwrap();
                }
            } 
        }
    }
}