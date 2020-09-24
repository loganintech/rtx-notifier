#![feature(async_closure)]

use std::net::TcpStream;

use chrono::Local;
use native_tls::{self, TlsStream};
use serde::{Deserialize, Serialize};

use config::*;
use error::NotifyError;

mod config;
mod error;
mod mail;
mod scraping;
mod product;


#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Subscriber {
    service: Vec<String>,
    to_phone_number: String,
    active: bool,
}

pub struct Notifier {
    pub twilio: Option<twilio::Client>,
    pub imap: Option<imap::Session<TlsStream<TcpStream>>>,
    pub config: Config,
}

impl Notifier {
    pub fn daemon_mode(&self) -> bool {
        self.config.application_config.daemon_mode
    }

    pub fn config(&self) -> bool {
        self.config.application_config.should_open_browser
    }

    pub fn should_send_notification(&self) -> bool {
        self.config.application_config.last_notification_sent
            < (Local::now() - chrono::Duration::minutes(30))
    }

    pub fn active_subscribers(&self, key: String) -> Vec<&Subscriber> {
        self
            .config
            .subscribers
            .iter()
            // Filter the subscribers to only active subscribers that are subscribed to this provider
            .filter(|subscriber| {
                subscriber.active && subscriber.service.contains(&key)
            }).collect::<Vec<&Subscriber>>()
    }

    pub fn get_from_phone_number(&self) -> Option<&String> {
        self
            .config
            .application_config
            .from_phone_number
            .as_ref()
    }
}

#[tokio::main]
async fn main() -> Result<(), NotifyError> {
    // Get notifier instance and settings
    let mut notifier = get_notifier().await?;
    loop {
        let start = Local::now();
        if let Err(e) = run_bot(&mut notifier).await {
            eprintln!("Error occurred: {}", e);
        }
        let end = Local::now();
        // Get total runtime in seconds
        let runtime = (start - end).num_seconds();

        // If we're not in daemon mode, break out of this loop
        if !notifier.config.application_config.daemon_mode {
            break;
        }
        // Otherwise, delay for the rest of the 30 second cycle
        tokio::time::delay_for(std::time::Duration::from_secs(
            30u64.checked_sub(runtime as u64).unwrap_or(0),
        ))
            .await;
    }

    Ok(())
}

async fn run_bot(notifier: &mut Notifier) -> Result<(), NotifyError> {
    // Check the scraped websites
    let scraped_set = scraping::get_providers_from_scraping(notifier).await?;
    // Check the mail providers
    let set = mail::get_providers_from_mail(notifier).await?;

    // If the last time we sent a message was more recent than 30 minutes ago, don't try to send messages

    // Only send a message if we haven't sent one in the last 5 minutes
    for provider in set.iter().chain(scraped_set.iter()) {
        // If we found any providers, send the messages
        // If it results in an error print the error
        if let Err(e) = provider.process_found_in_stock_notification(notifier).await {
            eprintln!("Error: {}", e);
        } else {
            // If we don't have an error, update the last notification sent timer
            notifier.config.application_config.last_notification_sent = Local::now();
        }
    }

    // Once we've run through re-write our config
    write_config(notifier).await?;
    Ok(())
}
