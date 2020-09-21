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

    let mailbox = notifier
        .imap
        .select("INBOX")
        .map_err(|_| NotifyError::MailboxLoad)?;

    let selected = (mailbox.exists - 10..mailbox.exists)
        .map(|n| n.to_string())
        .collect::<Vec<String>>()
        .join(",");

    let messages = notifier
        .imap
        .fetch(
            selected,
            "(ENVELOPE BODY[] FLAGS INTERNALDATE BODY[HEADER])",
        )
        .map_err(|_| NotifyError::EmailFetch)?;


    let set = messages.into_iter().filter_map(|f| {
        let body = f.envelope()?;
        let subject = body.subject?;
        let subject = String::from_utf8(subject.to_vec()).ok()?;
        let subject = subject.to_ascii_lowercase();
        let date = f.internal_date()?;
        if subject.contains("evga") && date > notifier.config.last_seen_evga {
            notifier.config.last_seen_evga = Local::now();
            Some(ProviderType::Evga)
        } else if subject.contains("newegg") && date > notifier.config.last_seen_newegg {
            notifier.config.last_seen_newegg = Local::now();
            Some(ProviderType::NewEgg)
        } else if subject.contains("asus") && date > notifier.config.last_seen_asus {
            notifier.config.last_seen_asus = Local::now();
            Some(ProviderType::Asus)
        } else { None }
    }).collect::<HashSet<ProviderType>>();

    write_config(&mut notifier).await?;

    for provider in set {
        provider.process_provider(&mut notifier).await?;
    }

    Ok(())
}
