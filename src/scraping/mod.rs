mod newegg;
mod nvidia;

use crate::error::NotifyError;
use crate::provider::ProviderType;
use crate::Notifier;

use scraper::{Html, Selector};
use serde::{Deserialize, Serialize};

use std::collections::HashSet;

#[derive(Serialize, Deserialize, Debug)]
pub struct ProductPage {
    pub id: i32,
    pub product_key: String,
    pub product: String,
    pub page: String,
    pub css_selector: String,
}

impl ProductPage {
    async fn is_available(&self) -> Result<ProviderType, NotifyError> {
        match self.product_key.as_str() {
            "nvidia" => Err(NotifyError::NoProductFound),
            "newegg" | "neweggrtx" => newegg::newegg_availability(self).await,
            _ => default_availability(self).await,
        }
    }
}

pub async fn get_providers_from_scraping(
    notifier: &mut Notifier,
) -> Result<HashSet<ProviderType>, NotifyError> {
    let rows = notifier
        .db
        .query("SELECT * FROM productpage WHERE active = true", &[])
        .await
        .map_err(|e| NotifyError::DBProductPageSelect(e))?;
    if rows.len() == 0 {
        return Ok(HashSet::new());
    }

    let pages: Vec<ProductPage> =
        serde_postgres::from_rows(&rows).map_err(|e| NotifyError::ProductPageFromRows(e))?;

    let mut providers = vec![];
    for page in pages {
        if let Ok(page) = page.is_available().await {
            providers.push(page);
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
        Selector::parse(&provider.css_selector).map_err(|_| NotifyError::HTMLParseFailed)?;
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
