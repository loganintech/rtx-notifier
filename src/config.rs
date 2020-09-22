use std::net::TcpStream;

use imap::{self, Session};
use native_tls::{self, TlsStream};
use tokio::fs::File;
use tokio::io::{AsyncReadExt, AsyncWriteExt};

use crate::{error::NotifyError, Config, Notifier};

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
    let mut file = File::open("./config.json")
        .await
        .map_err(|e| NotifyError::ConfigLoad(e))?;

    let mut buf = String::new();
    file.read_to_string(&mut buf)
        .await
        .map_err(|e| NotifyError::ConfigLoad(e))?;

    let config: Config = serde_json::from_str(&buf).map_err(|e| NotifyError::ConfigParse(e))?;

    let imap = if config.application_config.imap_host.is_none()
        || config.application_config.imap_username.is_none()
        || config.application_config.imap_password.is_none()
        || config.application_config.from_phone_number.is_none()
    {
        None
    } else {
        Some(get_imap(
            &config.application_config.imap_host.clone().unwrap(),
            &config.application_config.imap_username.clone().unwrap(),
            &config.application_config.imap_password.clone().unwrap(),
        )?)
    };

    let twilio = if config.application_config.twilio_account_id.is_none()
        || config.application_config.twilio_auth_token.is_none()
    {
        None
    } else {
        Some(twilio::Client::new(
            &config.application_config.twilio_account_id.clone().unwrap(),
            &config.application_config.twilio_auth_token.clone().unwrap(),
        ))
    };

    Ok(Notifier {
        imap,
        twilio,
        config,
    })
}

pub async fn write_config(notifier: &mut Notifier) -> Result<(), NotifyError> {
    let mut file = File::create("./config.json")
        .await
        .map_err(|_| NotifyError::ConfigUpdate)?;

    file.write_all(
        serde_json::to_string_pretty(&notifier.config)
            .map_err(|_| NotifyError::ConfigUpdate)?
            .as_bytes(),
    )
    .await
    .map_err(|_| NotifyError::ConfigUpdate)
}
