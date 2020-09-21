#![feature(async_closure)]

use std::collections::HashSet;
use std::net::TcpStream;

use chrono::{DateTime, Local};
use imap;
use native_tls::{self, TlsStream};
use serde::{Deserialize, Serialize};

use config::*;
use error::NotifyError;
use provider::ProviderType;

mod error;
mod config;
mod provider;
mod mail;
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

    let mut set = mail::get_providers_from_mail(&mut notifier).await?;
    let mut scraped_set = scraping::get_providers_from_scraping(&mut notifier).await?;

    for provider in set {
        provider.process_provider(&mut notifier).await?;
    }

    for provider in dbg!(scraped_set) {
        provider.process_provider(&mut notifier).await?;
    }

    write_config(&mut notifier).await?;
    Ok(())
}
