mod newegg;
mod bestbuy;

use crate::error::NotifyError;
use crate::provider::ProviderType;
use crate::Notifier;

use scraper::{Html, Selector};
use serde::{Deserialize, Serialize};

use std::collections::HashSet;

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct ProductPage {
    pub product_key: String,
    pub product: String,
    pub page: String,
    pub css_selector: Option<String>,
}

impl ProductPage {
    async fn is_available(&self) -> Result<ProviderType, NotifyError> {
        match self.product_key.as_str() {
            "nvidia" => Err(NotifyError::NoProductFound),
            "newegg" | "neweggrtx" => newegg::newegg_availability(self).await,
            "bestbuy" => bestbuy::bestbuy_availability(self).await,
            _ => default_availability(self).await,
        }
    }
}

pub async fn get_providers_from_scraping(
    notifier: &mut Notifier,
) -> Result<HashSet<ProviderType>, NotifyError> {
    let mut futs = vec![];
    for page in &notifier.config.products {
        futs.push(page.is_available());
    }

    let joined = futures::future::join_all(futs).await;

    let mut providers = vec![];
    for res in joined {
        match res {
            Ok(res) => providers.push(res),
            Err(NotifyError::WebRequestFailed(e)) => eprintln!("{}", e),
            _ => {}
        }
    }


    Ok(providers.into_iter().collect::<HashSet<ProviderType>>())
}

pub async fn default_availability(provider: &ProductPage) -> Result<ProviderType, NotifyError> {
    let resp = reqwest::get(&provider.page)
        .await
        .map_err(|_| NotifyError::HTMLParseFailed)?
        .text()
        .await
        .map_err(|_| NotifyError::HTMLParseFailed)?;

    let document = Html::parse_document(&resp);

    let selector =
        Selector::parse(&provider.css_selector.clone().unwrap_or("".to_string())).map_err(|_| NotifyError::HTMLParseFailed)?;
    let mut selected = document.select(&selector);
    let found = selected.next();
    if found.is_none()
        || (found.is_some()
            && !found
                .unwrap()
                .inner_html()
                .to_ascii_lowercase()
                .contains("out of stock"))
    {
        if let Some(provider) = ProviderType::from_product(
            &provider.product_key,
            provider.product.clone(),
            provider.page.clone(),
        ) {
            return Ok(provider);
        }
    }

    Err(NotifyError::NoProductFound)
}
