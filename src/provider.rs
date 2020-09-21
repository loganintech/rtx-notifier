use crate::error::NotifyError;
use crate::{Notifier, Subscriber};
use twilio::OutboundMessage;

#[derive(Eq, PartialEq, Clone, Hash, Debug)]
pub enum EvgaProduct {
    Known(String, String),
    Unknown,
}

#[derive(Eq, PartialEq, Clone, Hash, Debug)]
pub enum NeweggProduct {
    Known(String, String),
    Unknown,
}

#[derive(Eq, PartialEq, Clone, Hash, Debug)]
pub enum ProviderType {
    Evga(EvgaProduct),
    NewEgg(NeweggProduct),
    FE(String, String),
}

impl ProviderType {
    pub async fn process_provider(&self, notifier: &mut Notifier) -> Result<(), NotifyError> {
        let provider: &'static str = self.to_key();
        let rows = notifier
            .db
            .query(
                "SELECT * FROM subscriber WHERE service = $1 AND active = true",
                &[&provider],
            )
            .await
            .map_err(|_| NotifyError::DBSubscriberSelect)?;

        if rows.len() == 0 {
            return Ok(());
        }

        let subscribers: Vec<Subscriber> =
            serde_postgres::from_rows(&rows).map_err(|e| NotifyError::SubscriberFromRows(e))?;

        for subscriber in subscribers {
            let message = self.new_stock_message();
            notifier
                .twilio
                .send_message(OutboundMessage::new(
                    &notifier.config.from_phone_number,
                    &subscriber.to_phone_number,
                    &message,
                ))
                .await
                .map_err(|e| NotifyError::TwilioSend(e))?;

            println!(
                "Sent [{}] message to {}",
                &message, subscriber.to_phone_number
            );
        }

        Ok(())
    }
}

impl ProviderType {
    fn to_key(&self) -> &'static str {
        use ProviderType::*;
        match self {
            Evga(_) => "evga",
            NewEgg(_) => "newegg",
            FE(_, _) => "nvidia",
        }
    }

    pub fn from_product(key: &str, name: String, page: String) -> Option<Self> {
        match key {
            "evgartx" => Some(ProviderType::Evga(EvgaProduct::Known(name, page))),
            "evga" => Some(ProviderType::Evga(EvgaProduct::Unknown)),
            "neweggrtx" => Some(ProviderType::NewEgg(NeweggProduct::Known(name, page))),
            "newegg" => Some(ProviderType::NewEgg(NeweggProduct::Unknown)),
            "nvidia" => Some(ProviderType::FE(name, page)),
            _ => None,
        }
    }

    fn new_stock_message(&self) -> String {
        match self {
            ProviderType::Evga(EvgaProduct::Known(name, page)) => {
                format!("EVGA has new {} for sale at {}!", name, page)
            }
            ProviderType::Evga(EvgaProduct::Unknown) => format!("EVGA has new products!"),
            ProviderType::NewEgg(NeweggProduct::Known(name, page)) => {
                format!("NewEgg has new {} for sale at {}", name, page)
            }
            ProviderType::NewEgg(NeweggProduct::Unknown) => format!("NewEgg has new products!"),
            ProviderType::FE(name, page) => format!("Nvidia has {} for sale at {}!", name, page),
        }
    }
}
