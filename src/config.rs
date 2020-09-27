use std::net::TcpStream;

use chrono::{DateTime, Local};
use imap::{self, Session};
use native_tls::{self, TlsStream};
use serde::{Deserialize, Serialize};
use tokio::fs::File;
use tokio::io::{AsyncReadExt, AsyncWriteExt};

use crate::{error::NotifyError, Notifier};
use crate::product::Product;
use crate::Subscriber;

const CONFIG_FILE_PATH: &str = "./config.json";

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct Config {
    pub application_config: ApplicationConfig,
    pub subscribers: Vec<Subscriber>,
    pub products: Vec<Product>,
}

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct ApplicationConfig {
    pub last_seen_evga: DateTime<Local>,
    pub last_seen_newegg: DateTime<Local>,
    pub last_seen_asus: DateTime<Local>,
    pub last_notification_sent: DateTime<Local>,
    pub twilio_auth_token: Option<String>,
    pub twilio_account_id: Option<String>,
    pub imap_username: Option<String>,
    pub imap_password: Option<String>,
    pub imap_host: Option<String>,
    pub from_phone_number: Option<String>,
    pub should_open_browser: bool,
    pub daemon_mode: bool,
    pub discord_url: Option<String>,
    pub scraping_timeout: Option<DateTime<Local>>,
}

impl ApplicationConfig {
    pub fn should_send_notification(&self) -> bool {
        self.last_notification_sent
            < (Local::now() - chrono::Duration::minutes(30))
    }

    pub fn has_twilio_config(&self) -> bool {
        self.twilio_account_id.is_some()
            && self.twilio_auth_token.is_some()
            && self.from_phone_number.is_some()
    }

    pub fn has_imap_config(&self) -> bool {
        self.imap_host.is_some()
            && self.imap_username.is_some()
            && self.imap_password.is_some()
    }

    pub fn should_open_browser(&self) -> bool {
        self.should_open_browser
    }

    pub fn should_scrape(&self) -> bool {
        match self.scraping_timeout {
            Some(timeout) if timeout < chrono::Local::now() => true,
            _ => false,
        }
    }
}

impl Notifier {
    pub async fn new() -> Result<Self, NotifyError> {
        // Open our config
        let mut file = File::open(CONFIG_FILE_PATH)
            .await
            .map_err(NotifyError::ConfigLoad)?;

        // And read it into a string
        let mut buf = String::new();
        file.read_to_string(&mut buf)
            .await
            .map_err(NotifyError::ConfigLoad)?;

        // Use serde to deserialize the config
        let config: Config = serde_json::from_str(&buf).map_err(NotifyError::ConfigParse)?;

        // If the imap config exists, get the imap session
        let imap = if config.application_config.has_imap_config() {
            Some(get_imap(
                &config.application_config.imap_host.as_ref().unwrap(),
                &config.application_config.imap_username.as_ref().unwrap(),
                &config.application_config.imap_password.as_ref().unwrap(),
            )?)
        } else {
            None
        };

        // If we have a twilio config create a client
        let twilio = if config.application_config.has_twilio_config() {
            Some(twilio::Client::new(
                &config
                    .application_config
                    .twilio_account_id
                    .as_ref()
                    .unwrap(),
                &config
                    .application_config
                    .twilio_auth_token
                    .as_ref()
                    .unwrap(),
            ))
        } else {
            None
        };

        // And return our built notifier
        Ok(Notifier {
            imap,
            twilio,
            config,
        })
    }

    pub fn daemon_mode(&self) -> bool {
        self.config.application_config.daemon_mode
    }
}

pub fn get_imap(
    host: &str,
    username: &str,
    password: &str,
) -> Result<Session<TlsStream<TcpStream>>, NotifyError> {
    let tls = native_tls::TlsConnector::builder()
        .build()
        .map_err(|_| NotifyError::TlsCreation)?;

    let client = imap::connect((host, 993), host, &tls)
        .map_err(|e| NotifyError::ImapConnection(Box::new(e)))?;

    client
        .login(username, password)
        .map_err(|_| NotifyError::ImapLogin)
}

pub async fn write_config(notifier: &mut Notifier) -> Result<(), NotifyError> {
    // Open the config file, creating it if it doesn't exist
    let mut file = File::create(CONFIG_FILE_PATH)
        .await
        .map_err(|_| NotifyError::ConfigUpdate)?;

    // Write the config file json back in a pretty editable format
    file.write_all(
        serde_json::to_string_pretty(&notifier.config)
            .map_err(|_| NotifyError::ConfigUpdate)?
            .as_bytes(),
    )
        .await
        .map_err(|_| NotifyError::ConfigUpdate)
}
