use std::collections::HashSet;

use async_trait::async_trait;

use crate::error::NotifyError;
use crate::Notifier;
use crate::product::{Product};

pub mod bestbuy;
pub mod newegg;
pub mod evga;

#[async_trait]
trait ScrapingProvider {
    async fn has_product() -> Result<Product, NotifyError>;
}

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
