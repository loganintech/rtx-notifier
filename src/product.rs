use std::process::Command;

use serde::{Deserialize, Serialize};
use twilio::OutboundMessage;

use crate::error::NotifyError;
use crate::scraping::{bestbuy, newegg, evga, *};
use crate::Notifier;
use crate::ProductDetails;

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct ProductPage {
    pub product_key: String,
    pub product: String,
    pub page: String,
    pub css_selector: Option<String>,
}

impl ProductPage {
    pub async fn is_available(&self) -> Result<Product, NotifyError> {
        match self.product_key.as_str() {
            "nvidia" => Err(NotifyError::NoProductFound),
            "newegg" | "neweggrtx" => newegg::newegg_availability(self).await,
            "bestbuy" => bestbuy::bestbuy_availability(self).await,
            "evga" => evga::evga_availability(self).await,
            _ => default_availability(self).await,
        }
    }
}

#[derive(Eq, PartialEq, Clone, Hash, Debug)]
pub enum Product {
    Evga(Option<ProductDetails>),
    NewEgg(Option<ProductDetails>),
    BestBuy(ProductDetails),
    FE(String, String),
}

impl Product {
    pub async fn process_found_in_stock_notification(
        &self,
        notifier: &mut Notifier,
    ) -> Result<(), NotifyError> {
        // If the twilio client is set
        if let Some(twilio) = &notifier.twilio {
            // Loop through all of our subscribers
            for subscriber in notifier
                .config
                .subscribers
                .iter()
                // Filter the subscribers to only active subscribers that are subscribed to this provider
                .filter(|subscriber| {
                    subscriber.active && subscriber.service.contains(&self.to_key().to_string())
                })
            {
                // Get the new in stock message for this provider
                let message = self.new_stock_message();
                // And send our text message
                twilio
                    .send_message(OutboundMessage::new(
                        // Get the from phone number. The unwrap code is supposedly unreachable because the twilio client being set
                        // is dependent on the fact that the from_phone_number is also set
                        notifier
                            .config
                            .application_config
                            .from_phone_number
                            .as_ref()
                            .unwrap_or(&"".to_string()),
                        // Get the phone number of our subscriber
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

        // If the notifier is configured to open this in a browser
        if notifier.config.application_config.should_open_browser {
            // Open the page in a browser
            self.open_in_browser()?;
        }

        Ok(())
    }

    // If we're using windows
    #[cfg(target_os = "windows")]
    fn open_in_browser(&self) -> Result<(), NotifyError> {
        // Get the url of the product
        let url = self.get_url()?;
        // Run the explorer command with the URL as the param
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

    // If we're on a mac
    #[cfg(target_os = "macos")]
    fn open_in_browser(&self) -> Result<(), NotifyError> {
        // Get the url of the product
        let url = self.get_url()?;
        // Run the explorer command with the URL as the param
        let mut child = Command::new("open")
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

    // If we're not on a mac or windows machine, just succeed without doing anything
    #[cfg(not(any(target_os = "macos", target_os = "windows")))]
    fn open_in_browser(&self) -> Result<(), NotifyError> {
        Ok(())
    }

    // Get the page from the Product
    fn get_url(&self) -> Result<&str, NotifyError> {
        // Get a reference to the page property of each product type
        match self {
            Product::Evga(Some(ProductDetails { page, .. }))
            | Product::NewEgg(Some(ProductDetails { page, .. }))
            | Product::BestBuy(ProductDetails { page, .. })
            | Product::FE(_, page) => Ok(page),
            _ => Err(NotifyError::NoPage),
        }
    }

    // Get the product key from the type
    fn to_key(&self) -> &'static str {
        use Product::*;
        match self {
            Evga(_) => "evga",
            NewEgg(_) => "newegg",
            FE(_, _) => "nvidia",
            BestBuy(_) => "bestbuy",
        }
    }

    // Get the product info from the key, name, and url
    pub fn from_product(key: &str, product: String, page: String) -> Option<Self> {
        match key {
            "evgartx" => Some(Product::Evga(Some(ProductDetails {
                product,
                page,
                ..ProductDetails::default()
            }))),
            "evga" => Some(Product::Evga(None)),
            "neweggrtx" => Some(Product::NewEgg(Some(ProductDetails {
                product,
                page,
                ..ProductDetails::default()
            }))),
            "newegg" => Some(Product::NewEgg(None)),
            "nvidia" => Some(Product::FE(product, page)),
            "bestbuy" => Some(Product::BestBuy(ProductDetails {
                product,
                page,
                ..ProductDetails::default()
            })),
            _ => None,
        }
    }

    // Get some new in stock messages depending on product type
    fn new_stock_message(&self) -> String {
        match self {
            Product::Evga(Some(ProductDetails { product, page, .. })) => {
                format!("EVGA has new {} for sale at {}!", product, page)
            }
            Product::Evga(None) => format!("EVGA has new products!"),
            Product::NewEgg(Some(ProductDetails { product, page, .. })) => {
                format!("NewEgg has new {} for sale at {}", product, page)
            }
            Product::NewEgg(None) => format!("NewEgg has new products!"),
            Product::FE(name, page) => format!("Nvidia has {} for sale at {}!", name, page),
            Product::BestBuy(ProductDetails { product, page, .. }) => {
                format!("Bestbuy has {} for sale at {}!", product, page)
            }
        }
    }
}
