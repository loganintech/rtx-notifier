use std::collections::HashSet;

use scraper::{Html, Selector};
use serde::Serialize;

use crate::error::NotifyError;
use crate::product::{Product, ProductPage};
use crate::Notifier;

pub mod bestbuy;
pub mod newegg;
pub mod evga;

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

pub async fn default_availability(_: &ProductPage) -> Result<Product, NotifyError> {
    Err(NotifyError::NoProductFound)
}
