use std::net::TcpStream;

use serde::{Serialize, Deserialize};
use chrono::{DateTime, Local};
use imap::{self, Session};
use native_tls::{self, TlsStream};
use tokio::fs::File;
use tokio::io::{AsyncReadExt, AsyncWriteExt};

use crate::{error::NotifyError, Notifier};
use crate::product::Product;
use crate::Subscriber;

const CONFIG_FILE_PATH: &'static str = "./config.json";

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
}

pub fn get_imap(
    host: &str,
    username: &str,
    password: &str,
) -> Result<Session<TlsStream<TcpStream>>, NotifyError> {
    let tls = native_tls::TlsConnector::builder()
        .build()
        .map_err(|_| NotifyError::TlsCreation)?;

    let client =
        imap::connect((host, 993), host, &tls).map_err(|e| NotifyError::ImapConnection(e))?;

    client
        .login(username, password)
        .map_err(|_| NotifyError::ImapLogin)
}

pub async fn get_notifier() -> Result<Notifier, NotifyError> {
    // Open our config
    let mut file = File::open(CONFIG_FILE_PATH)
        .await
        .map_err(|e| NotifyError::ConfigLoad(e))?;

    // And read it into a string
    let mut buf = String::new();
    file.read_to_string(&mut buf)
        .await
        .map_err(|e| NotifyError::ConfigLoad(e))?;

    // Use serde to deserialize the config
    let config: Config = serde_json::from_str(&buf).map_err(|e| NotifyError::ConfigParse(e))?;

    // If the config is missing any of the imap properties, set imap to None
    let imap = if config.application_config.imap_host.is_none()
               || config.application_config.imap_username.is_none()
               || config.application_config.imap_password.is_none()
    {
        None
    } else {
        // Otherwise start an IMAP session
        Some(get_imap(
            &config.application_config.imap_host.clone().unwrap(),
            &config.application_config.imap_username.clone().unwrap(),
            &config.application_config.imap_password.clone().unwrap(),
        )?)
    };

    // If any of our twilio config is missing, set it to none
    let twilio = if config.application_config.twilio_account_id.is_none()
                 || config.application_config.twilio_auth_token.is_none()
                 || config.application_config.from_phone_number.is_none() {
        None
    } else {
        // Otherwise create a twilio client
        Some(twilio::Client::new(
            &config.application_config.twilio_account_id.clone().unwrap(),
            &config.application_config.twilio_auth_token.clone().unwrap(),
        ))
    };

    // And return our built notifier
    Ok(Notifier {
        imap,
        twilio,
        config,
    })
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
