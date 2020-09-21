use std::net::TcpStream;

use imap::{self, Session};
use native_tls::{self, TlsStream};
use tokio_postgres::NoTls;

use crate::{error::NotifyError, Notifier, ProjectConfig};

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
    let (client, con) = tokio_postgres::connect(
        "host=localhost user=postgres dbname=projectnotifier password=projectnotifpass sslmode=disable port=5433",
        NoTls,
    )
        .await
        .map_err(|_| NotifyError::DBConnection)?;

    tokio::spawn(async move {
        if let Err(e) = con.await {
            eprintln!("connection error: {}", e);
        }
    });

    let config_rows = client
        .query(
            "SELECT id, \
         last_seen_evga, \
         last_seen_newegg, \
         last_seen_asus, \
         twilio_auth_token, \
         twilio_account_id, \
         imap_username, \
         imap_password, \
         imap_host, \
         from_phone_number, \
         last_notification_sent \
         FROM config ORDER BY id DESC LIMIT 1",
            &[],
        )
        .await
        .map_err(|e| NotifyError::DBConfigSelect(e))?;

    let mut configs = vec![];
    for row in config_rows {
        configs.push(ProjectConfig {
            id: row.get(0),
            last_seen_evga: row.get(1),
            last_seen_newegg: row.get(2),
            last_seen_asus: row.get(3),
            twilio_auth_token: row.get(4),
            twilio_account_id: row.get(5),
            imap_username: row.get(6),
            imap_password: row.get(7),
            imap_host: row.get(8),
            from_phone_number: row.get(9),
            last_notification_sent: row.get(10),
        });
    }

    if let Some(config) = configs.into_iter().next() {
        Ok(Notifier {
            db: client,
            imap: get_imap(
                &config.imap_host,
                &config.imap_username,
                &config.imap_password,
            )?,
            twilio: twilio::Client::new(&config.twilio_account_id, &config.twilio_auth_token),
            config,
        })
    } else {
        Err(NotifyError::DBNoConfigFound)
    }
}

pub async fn write_config(notifier: &mut Notifier) -> Result<(), NotifyError> {
    notifier
        .db
        .execute(
            "UPDATE config SET \
                 last_seen_evga = $1,\
                 last_seen_newegg = $2,\
                 last_seen_asus = $3,\
                 twilio_auth_token = $4,\
                 twilio_account_id = $5,\
                 imap_username = $6,\
                 imap_password = $7,\
                 imap_host = $8,\
                 from_phone_number = $9,\
                 last_notification_sent = $10\
             WHERE id = $11",
            &[
                &notifier.config.last_seen_evga,
                &notifier.config.last_seen_newegg,
                &notifier.config.last_seen_asus,
                &notifier.config.twilio_auth_token,
                &notifier.config.twilio_account_id,
                &notifier.config.imap_username,
                &notifier.config.imap_password,
                &notifier.config.imap_host,
                &notifier.config.from_phone_number,
                &notifier.config.last_notification_sent,
                &notifier.config.id,
            ],
        )
        .await
        .map_err(|_| NotifyError::ConfigUpdate)?;

    Ok(())
}
