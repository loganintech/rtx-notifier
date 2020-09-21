use crate::{Notifier, Subscriber};
use crate::error::NotifyError;
use twilio::OutboundMessage;

#[derive(Eq, PartialEq, Copy, Clone, Hash, Debug)]
pub enum ProviderType {
    Evga,
    Asus,
    NewEgg,
}

impl ProviderType {
    pub async fn process_provider(&self, notifier: &mut Notifier) -> Result<(), NotifyError> {
        let provider: &'static str = self.into();
        let rows = notifier
            .db
            .query("SELECT * FROM subscriber WHERE service = $1", &[&provider])
            .await
            .map_err(|_| NotifyError::DBSubscriberSelect)?;

        if rows.len() == 0 {
            return Ok(());
        }

        let subscribers: Vec<Subscriber> = serde_postgres::from_rows(&rows)
            .map_err(|e| NotifyError::SubscriberFromRows(e))?;

        for subscriber in subscribers {
            let message = format!("{} has new stock!", provider.to_ascii_uppercase());
            notifier
                .twilio
                .send_message(OutboundMessage::new(
                    &notifier.config.from_phone_number,
                    &subscriber.to_phone_number,
                    &message
                ))
                .await
                .map_err(|e| NotifyError::TwilioSend(e))?;

            println!("Sent [{}] message to {}", &message, subscriber.to_phone_number);
        }

        Ok(())
    }
}

impl From<&ProviderType> for &'static str {
    fn from(provider: &ProviderType) -> &'static str {
        use ProviderType::*;
        match provider {
            Evga => {"evga"},
            Asus => {"asus"},
            NewEgg => {"newegg"},
        }
    }
}

