#![feature(async_closure)]

use std::net::TcpStream;

use chrono::{DateTime, Local};
use imap;
use native_tls::{self, TlsStream};
use serde::{Deserialize, Serialize};

use config::*;
use error::NotifyError;

mod config;
mod error;
mod mail;
mod provider;
mod scraping;

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct ProjectConfig {
    id: i32,
    last_seen_evga: DateTime<Local>,
    last_seen_newegg: DateTime<Local>,
    last_seen_asus: DateTime<Local>,
    twilio_auth_token: String,
    twilio_account_id: String,
    imap_username: String,
    imap_password: String,
    imap_host: String,
    from_phone_number: String,
    last_notification_sent: DateTime<Local>,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct Subscriber {
    id: i32,
    service: String,
    to_phone_number: String,
}

pub struct Notifier {
    pub db: tokio_postgres::Client,
    pub twilio: twilio::Client,
    pub imap: imap::Session<TlsStream<TcpStream>>,
    pub config: ProjectConfig,
}

#[tokio::main]
async fn main() -> Result<(), NotifyError> {
    let mut notifier = get_notifier().await?;

    let set = mail::get_providers_from_mail(&mut notifier).await?;
    let scraped_set = scraping::get_providers_from_scraping(&mut notifier).await?;

    if notifier.config.last_notification_sent < (Local::now() - (chrono::Duration::minutes(5))) {
        // Only send a message if we haven't sent one in the last 5 minutes
        for provider in set.iter().chain(scraped_set.iter()) {
            if let Err(e) = provider.process_provider(&mut notifier).await {
                eprintln!("Error: {}", e);
            } else {
                notifier.config.last_notification_sent = Local::now();
            }
        }
    }

    write_config(&mut notifier).await?;
    Ok(())
}
