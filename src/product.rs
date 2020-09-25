use std::process::Command;

use serde::{Deserialize, Serialize};

use crate::{
    error::NotifyError,
    notifier::{discord, twilio},
    scraping::{
        bestbuy::BestBuyScraper, evga::EvgaScraper, newegg::NeweggScraper, ScrapingProvider,
    },
    Notifier,
};

// This is a workaround for Serde because it doesn't support literals as defaults
#[allow(non_snake_case)]
const fn TRUE() -> bool {
    true
}

#[derive(Serialize, Deserialize, Clone, Debug, Eq, PartialEq, Default, Hash)]
pub struct ProductDetails {
    pub product: String,
    pub page: String,
    pub product_key: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub css_selector: Option<String>,
    #[serde(default = "TRUE")]
    pub active: bool,
}

impl ProductDetails {
    pub async fn is_available(&self) -> Result<Product, NotifyError> {
        match self.product_key.as_ref() {
            "newegg" | "neweggrtx" => NeweggScraper.is_available(self).await,
            "bestbuy" => BestBuyScraper.is_available(self).await,
            "evga" | "evgartx" => EvgaScraper.is_available(self).await,
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
    pub async fn handle_found_product(&self, notifier: &mut Notifier) -> Result<(), NotifyError> {
        // If the notifier is configured to open this in a browser
        if notifier.config.should_open_browser() {
            // Open the page in a browser
            self.open_in_browser()?;
        }

        if let Some(discord_url) = &notifier.config.application_config.discord_url {
            discord::send_webhook(self, discord_url).await?
        }

        let has_twilio_config = notifier.config.has_twilio_config();
        if has_twilio_config && notifier.config.should_send_notification() {
            let subscribers = notifier.active_subscribers(self.to_key());
            let client = notifier.twilio.as_ref().unwrap();
            for subscriber in subscribers {
                twilio::send_twilio_message(
                    self,
                    client,
                    subscriber,
                    notifier
                        .config
                        .application_config
                        .from_phone_number
                        .as_ref()
                        .unwrap(),
                )
                .await?;
            }
        }

        Ok(())
    }

    fn run_command(&self, command: &str, args: &[&str]) -> Result<(), NotifyError> {
        // Get the url of the product.rs
        let url = self.get_url()?;
        // Run the explorer command with the URL as the param
        let mut child = Command::new(command)
            .args(args)
            .spawn()
            .map_err(NotifyError::CommandErr)?;
        let res = child.wait().map_err(NotifyError::CommandErr)?;
        if res.success() || res.code() == Some(1) {
            Ok(())
        } else {
            Err(NotifyError::CommandResult(res.code().unwrap_or(0)))
        }
    }

    // If we're using windows
    #[cfg(target_os = "windows")]
    fn open_in_browser(&self) -> Result<(), NotifyError> {
        let url = self.get_url()?;
        self.run_command("cmd", &["/C", "start", url])
    }

    // If we're on a mac
    #[cfg(target_os = "macos")]
    fn open_in_browser(&self) -> Result<(), NotifyError> {
        let url = self.get_url()?;
        self.run_command("open", &[url])
    }

    // If we're not on a mac or windows machine, just succeed without doing anything
    #[cfg(not(any(target_os = "macos", target_os = "windows")))]
    fn open_in_browser(&self) -> Result<(), NotifyError> {
        Ok(())
    }

    // Get the page from the Product
    pub fn get_url(&self) -> Result<&str, NotifyError> {
        // Get a reference to the page property of each product.rs type
        match self {
            Product::Evga(Some(ProductDetails { page, .. }))
            | Product::NewEgg(Some(ProductDetails { page, .. }))
            | Product::BestBuy(ProductDetails { page, .. })
            | Product::FE(_, page) => Ok(page),
            _ => Err(NotifyError::NoPage),
        }
    }

    // Get the product.rs key from the type
    pub fn to_key(&self) -> &'static str {
        use Product::*;
        match self {
            Evga(_) => "evga",
            NewEgg(_) => "newegg",
            FE(_, _) => "nvidia",
            BestBuy(_) => "bestbuy",
        }
    }

    // Get the product.rs info from the key, name, and url
    pub fn from_product(key: &str, product: String, page: String) -> Option<Self> {
        match key {
            "evgartx" => Some(Product::Evga(Some(
                ProductDetails::new_from_product_and_page(product, page),
            ))),
            "neweggrtx" => Some(Product::NewEgg(Some(
                ProductDetails::new_from_product_and_page(product, page),
            ))),
            "bestbuy" => Some(Product::BestBuy(ProductDetails::new_from_product_and_page(
                product, page,
            ))),
            "evga" => Some(Product::Evga(None)),
            "newegg" => Some(Product::NewEgg(None)),
            "nvidia" => Some(Product::FE(product, page)),
            _ => None,
        }
    }

    // Get some new in stock messages depending on product.rs type
    pub fn new_stock_message(&self) -> String {
        match self {
            Product::Evga(Some(ProductDetails { product, page, .. })) => {
                format!("EVGA has new {} for sale at {}!", product, page)
            }
            Product::NewEgg(Some(ProductDetails { product, page, .. })) => {
                format!("NewEgg has new {} for sale at {}", product, page)
            }
            Product::BestBuy(ProductDetails { product, page, .. }) => {
                format!("Bestbuy has {} for sale at {}!", product, page)
            }
            Product::FE(name, page) => format!("Nvidia has {} for sale at {}!", name, page),
            Product::Evga(None) => "EVGA has new products!".to_string(),
            Product::NewEgg(None) => "NewEgg has new products!".to_string(),
        }
    }
}
