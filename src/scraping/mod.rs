use std::collections::HashSet;

use scraper::{Html, Selector};
use serde::Serialize;

use crate::error::NotifyError;
use crate::product::{Product, ProductPage};
use crate::Notifier;

pub mod bestbuy;
pub mod newegg;

pub async fn get_providers_from_scraping(
    notifier: &mut Notifier,
) -> Result<HashSet<Product>, NotifyError> {
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

    Ok(providers.into_iter().collect::<HashSet<Product>>())
}

pub async fn default_availability(provider: &ProductPage) -> Result<Product, NotifyError> {
    let resp = reqwest::get(&provider.page)
        .await
        .map_err(|_| NotifyError::HTMLParseFailed)?
        .text()
        .await
        .map_err(|_| NotifyError::HTMLParseFailed)?;

    let document = Html::parse_document(&resp);

    let selector = Selector::parse(&provider.css_selector.clone().unwrap_or("".to_string()))
        .map_err(|_| NotifyError::HTMLParseFailed)?;
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
        if let Some(provider) = Product::from_product(
            &provider.product_key,
            provider.product.clone(),
            provider.page.clone(),
        ) {
            return Ok(provider);
        }
    }

    Err(NotifyError::NoProductFound)
}
