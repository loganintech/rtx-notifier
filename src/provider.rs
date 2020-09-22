use std::process::Command;

use twilio::OutboundMessage;

use crate::error::NotifyError;
use crate::Notifier;
use crate::Product;

#[derive(Eq, PartialEq, Clone, Hash, Debug)]
pub enum ProviderType {
    Evga(Option<Product>),
    NewEgg(Option<Product>),
    BestBuy(Product),
    FE(String, String),
}

impl ProviderType {
    pub async fn process_provider(&self, notifier: &mut Notifier) -> Result<(), NotifyError> {
        if let Some(twilio) = &notifier.twilio {
            for subscriber in notifier
                .config
                .subscribers
                .iter()
                .filter(|p| p.active && p.service.contains(&self.to_key().to_string()))
            {
                let message = self.new_stock_message();
                twilio
                    .send_message(OutboundMessage::new(
                        notifier.config.application_config.from_phone_number.as_ref().unwrap_or(&"".to_string()),
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
        }

        if notifier.config.application_config.should_open_browser {
            self.open_in_browser()?;
        }

        Ok(())
    }

    #[cfg(target_os = "windows")]
    fn open_in_browser(&self) -> Result<(), NotifyError> {
        let url = self.get_url()?;
        let mut child = Command::new("explorer.exe")
            .arg(url)
            .spawn()
            .map_err(|e| NotifyError::CommandErr(e))?;
        let res = child.wait().map_err(|e| NotifyError::CommandErr(e))?;
        if res.success() {
            Ok(())
        } else {
            Err(NotifyError::CommandResult(res.code().unwrap_or(0)))
        }
    }

    #[cfg(target_os = "macos")]
    fn open_in_browser(&self) -> Result<(), NotifyError> {
        let url = self.get_url()?;
        Command::new("open").arg(url)
    }

    #[cfg(not(any(target_os = "macos", target_os = "windows")))]
    fn open_in_browser(&self) -> Result<(), NotifyError> {
        Ok(())
    }

    fn get_url(&self) -> Result<&str, NotifyError> {
        match self {
            ProviderType::Evga(Some(Product { page, .. }))
            | ProviderType::NewEgg(Some(Product { page, .. }))
            | ProviderType::BestBuy(Product { page, .. })
            | ProviderType::FE(_, page) => Ok(page),
            _ => Err(NotifyError::NoPage),
        }
    }

    fn to_key(&self) -> &'static str {
        use ProviderType::*;
        match self {
            Evga(_) => "evga",
            NewEgg(_) => "newegg",
            FE(_, _) => "nvidia",
            BestBuy(_) => "bestbuy",
        }
    }

    pub fn from_product(key: &str, product: String, page: String) -> Option<Self> {
        match key {
            "evgartx" => Some(ProviderType::Evga(Some(Product {
                product,
                page,
                ..Product::default()
            }))),
            "evga" => Some(ProviderType::Evga(None)),
            "neweggrtx" => Some(ProviderType::NewEgg(Some(Product {
                product,
                page,
                ..Product::default()
            }))),
            "newegg" => Some(ProviderType::NewEgg(None)),
            "nvidia" => Some(ProviderType::FE(product, page)),
            "bestbuy" => Some(ProviderType::BestBuy(Product {
                product,
                page,
                ..Product::default()
            })),
            _ => None,
        }
    }

    fn new_stock_message(&self) -> String {
        match self {
            ProviderType::Evga(Some(Product { product, page, .. })) => {
                format!("EVGA has new {} for sale at {}!", product, page)
            }
            ProviderType::Evga(None) => format!("EVGA has new products!"),
            ProviderType::NewEgg(Some(Product { product, page, .. })) => {
                format!("NewEgg has new {} for sale at {}", product, page)
            }
            ProviderType::NewEgg(None) => format!("NewEgg has new products!"),
            ProviderType::FE(name, page) => format!("Nvidia has {} for sale at {}!", name, page),
            ProviderType::BestBuy(Product { product, page, .. }) => {
                format!("Bestbuy has {} for sale at {}!", product, page)
            }
        }
    }
}
