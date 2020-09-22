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

    Ok(Notifier {
        imap: get_imap(
            &config.application_config.imap_host,
            &config.application_config.imap_username,
            &config.application_config.imap_password,
        )?,
        twilio: twilio::Client::new(
            &config.application_config.twilio_account_id,
            &config.application_config.twilio_auth_token,
        ),
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
