use std::process::Command;

use serde::{Deserialize, Serialize};
use twilio::OutboundMessage;

use crate::error::NotifyError;
use crate::Notifier;
use crate::scraping::{bestbuy, evga, newegg};

#[allow(non_snake_case)]
const fn FALSE() -> bool { false }

#[derive(Serialize, Deserialize, Clone, Debug, Eq, PartialEq, Default, Hash)]
pub struct ProductDetails {
    pub product: String,
    pub page: String,
    pub product_key: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub css_selector: Option<String>,
    #[serde(default="FALSE")]
    pub active: bool,
}

impl ProductDetails {
    pub async fn is_available(&self) -> Result<Product, NotifyError> {
        match self.product_key.as_ref() {
            "newegg" => newegg::newegg_availability(self).await,
            "bestbuy" => bestbuy::bestbuy_availability(self).await,
            "evga" => evga::evga_availability(self).await,
            _ => Err(NotifyError::NoProductFound),
        }
    }
}

impl ProductDetails {
    pub fn new_from_product_and_page(product: String, page: String) -> Self {
        Self {
            product,
            page,
            ..Self::default()
        }
    }
}


#[derive(Eq, PartialEq, Clone, Hash, Debug, Serialize, Deserialize)]
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
        let config = &notifier.config;
        // If the twilio client is set
        if let Some(twilio) = &notifier.twilio {
            // Loop through all of our subscribers
            for subscriber in notifier.active_subscribers(self.to_key().to_string())
            {
                if config.should_send_notification() {
                    // Get the new in stock message for this provider
                    let message = self.new_stock_message();
                    // And send our text message
                    twilio
                        .send_message(OutboundMessage::new(
                            // Get the from phone number. The unwrap code is supposedly unreachable because the twilio client being set
                            // is dependent on the fact that the from_phone_number is also set
                            notifier.get_from_phone_number().unwrap_or(&"".to_string()),
                            // Get the phone number of our subscriber
                            &subscriber.to_phone_number,
                            &message,
                        ))
                        .await
                        .map_err(NotifyError::TwilioSend)?;

                    println!(
                        "Sent [{}] message to {}",
                        &message, subscriber.to_phone_number
                    );
                }
            }
        }

        // If the notifier is configured to open this in a browser
        if config.should_send_notification() {
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
            .map_err(NotifyError::CommandErr)?;
        let res = child.wait().map_err(NotifyError::CommandErr)?;
        if res.success() || res.code() == Some(1) {
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
            "evgartx" => Some(Product::Evga(Some(ProductDetails::new_from_product_and_page(product, page)))),
            "neweggrtx" => Some(Product::NewEgg(Some(ProductDetails::new_from_product_and_page(product, page)))),
            "bestbuy" => Some(Product::BestBuy(ProductDetails::new_from_product_and_page(product, page))),
            "evga" => Some(Product::Evga(None)),
            "newegg" => Some(Product::NewEgg(None)),
            "nvidia" => Some(Product::FE(product, page)),
            _ => None,
        }
    }

    // Get some new in stock messages depending on product type
    fn new_stock_message(&self) -> String {
        match self {
            Product::Evga(Some(ProductDetails { product, page, .. })) => format!("EVGA has new {} for sale at {}!", product, page),
            Product::NewEgg(Some(ProductDetails { product, page, .. })) => format!("NewEgg has new {} for sale at {}", product, page),
            Product::BestBuy(ProductDetails { product, page, .. }) => format!("Bestbuy has {} for sale at {}!", product, page),
            Product::FE(name, page) => format!("Nvidia has {} for sale at {}!", name, page),
            Product::Evga(None) => "EVGA has new products!".to_string(),
            Product::NewEgg(None) => "NewEgg has new products!".to_string(),
        }
    }
}
