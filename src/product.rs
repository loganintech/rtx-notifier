use std::process::Command;

use rand::{Rng, thread_rng};
use serde::{Deserialize, Serialize};

use crate::{
    error::NotifyError,
    scraping::{
        amazon::AmazonScraper, bestbuy::BestBuyScraper, bnh::BnHScraper, evga::EvgaScraper,
        newegg::NeweggScraper, ScrapingProvider,
    },
};


#[derive(Serialize, Deserialize, Clone, Debug, Eq, PartialEq, Default, Hash)]
pub struct ProductDetails {
    pub product: String,
    pub page: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub css_selector: Option<String>,
    pub active: Option<bool>,
    pub active_chance: Option<u8>,
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
#[serde(rename_all = "lowercase")]
pub enum Product {
    Evga(Option<ProductDetails>),
    NewEgg(Option<ProductDetails>),
    BestBuy(ProductDetails),
    Nvidia(ProductDetails),
    BnH(ProductDetails),
    Amazon(ProductDetails),
}

impl Product {
    pub async fn is_available(&self) -> Result<Product, NotifyError> {
        match self {
            Product::NewEgg(Some(_)) => NeweggScraper.is_available(self).await,
            Product::BestBuy(_) => BestBuyScraper.is_available(self).await,
            Product::Evga(Some(_)) => EvgaScraper.is_available(self).await,
            Product::BnH(_) => BnHScraper.is_available(self).await,
            Product::Amazon(_) => AmazonScraper.is_available(self).await,
            _ => Err(NotifyError::NoProductFound),
        }
    }

    fn run_command(&self, command: &str, args: &[&str]) -> Result<(), NotifyError> {
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
    pub fn open_in_browser(&self) -> Result<(), NotifyError> {
        let url = self.get_url()?;
        self.run_command("cmd", &["/C", "start", url])
    }

    // If we're on a mac
    #[cfg(target_os = "macos")]
    pub fn open_in_browser(&self) -> Result<(), NotifyError> {
        let url = self.get_url()?;
        self.run_command("open", &[url])
    }

    // If we're not on a mac or windows machine, just succeed without doing anything
    #[cfg(not(any(target_os = "macos", target_os = "windows")))]
    pub fn open_in_browser(&self) -> Result<(), NotifyError> {
        Ok(())
    }

    // Get the page from the Product
    pub fn get_url(&self) -> Result<&str, NotifyError> {
        // Get a reference to the page property of each product.rs type
        match self {
            Product::Evga(Some(ProductDetails { page, .. }))
            | Product::NewEgg(Some(ProductDetails { page, .. }))
            | Product::BestBuy(ProductDetails { page, .. })
            | Product::BnH(ProductDetails { page, .. })
            | Product::Amazon(ProductDetails { page, .. })
            | Product::Nvidia(ProductDetails { page, .. }) => Ok(page),
            _ => Err(NotifyError::NoPage),
        }
    }

    pub fn is_active(&self) -> bool {
        // Get a reference to the page property of each product.rs type
        let active = match self {
            Product::Evga(Some(ProductDetails { active: Some(active), .. }))
            | Product::NewEgg(Some(ProductDetails { active: Some(active), .. }))
            | Product::BestBuy(ProductDetails { active: Some(active), .. })
            | Product::Nvidia(ProductDetails { active: Some(active), .. })
            | Product::BnH(ProductDetails { active: Some(active), .. })
            | Product::Amazon(ProductDetails { active: Some(active), .. }) => {
                *active
            }
            _ => true,
        };

        let active_chance = match self {
            Product::Evga(Some(ProductDetails { active_chance: Some(active_chance), .. }))
            | Product::NewEgg(Some(ProductDetails { active_chance: Some(active_chance), .. }))
            | Product::BestBuy(ProductDetails { active_chance: Some(active_chance), .. })
            | Product::Nvidia(ProductDetails { active_chance: Some(active_chance), .. })
            | Product::BnH(ProductDetails { active_chance: Some(active_chance), .. })
            | Product::Amazon(ProductDetails { active_chance: Some(active_chance), .. }) => {
                *active_chance
            }
            _ => 10,
        };

        let chance = (active_chance / 10) as f64;
        let chance = if chance > 1.0 { 1.0 } else if chance <= 0.0 { 0.0 } else { chance };

        active && thread_rng().gen_bool(chance)
    }

    // Get the page from the Product
    pub fn get_css_selector(&self) -> Result<&str, NotifyError> {
        // Get a reference to the page property of each product.rs type
        match self {
            Product::Evga(Some(ProductDetails { css_selector: Some(css_selector), .. }))
            | Product::NewEgg(Some(ProductDetails { css_selector: Some(css_selector), .. }))
            | Product::BnH(ProductDetails { css_selector: Some(css_selector), .. })
            | Product::Amazon(ProductDetails { css_selector: Some(css_selector), .. })
            | Product::BestBuy(ProductDetails { css_selector: Some(css_selector), .. })
              => Ok(css_selector.as_str()),
            _ => Err(NotifyError::NoneCSSSelector),
        }
    }

    // Get the product.rs key from the type
    pub fn to_key(&self) -> &'static str {
        use Product::*;
        match self {
            Evga(_) => "evga",
            NewEgg(_) => "newegg",
            Nvidia(_) => "nvidia",
            BestBuy(_) => "bestbuy",
            BnH(_) => "bnh",
            Amazon(_) => "amazon",
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
            "nvidia" => Some(Product::Nvidia(ProductDetails::new_from_product_and_page(
                product, page,
            ))),
            "bnh" => Some(Product::BnH(ProductDetails::new_from_product_and_page(
                product, page,
            ))),
            "amazon" => Some(Product::Amazon(ProductDetails::new_from_product_and_page(
                product, page,
            ))),
            _ => None,
        }
    }

    // Get some new in stock messages depending on product.rs type
    pub fn new_stock_message(&self) -> String {
        match self {
            Product::Evga(Some(ProductDetails { product, page, .. })) => {
                format!("EVGA has new {} for sale at {}", product, page)
            }
            Product::NewEgg(Some(ProductDetails { product, page, .. })) => {
                format!("NewEgg has new {} for sale at {}", product, page)
            }
            Product::BestBuy(ProductDetails { product, page, .. }) => {
                format!("Bestbuy has {} for sale at {}", product, page)
            }
            Product::Nvidia(ProductDetails { product, page, .. }) => {
                format!("Nvidia has {} for sale at {}", product, page)
            }
            Product::BnH(ProductDetails { product, page, .. }) => {
                format!("BnH has {} for sale at {}", product, page)
            }
            Product::Amazon(ProductDetails { product, page, .. }) => {
                format!("Amazon has {} for sale at {}", product, page)
            }
            Product::Evga(None) => "EVGA has new products!".to_string(),
            Product::NewEgg(None) => "NewEgg has new products!".to_string(),
        }
    }
}
