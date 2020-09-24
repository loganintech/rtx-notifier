use std::collections::HashSet;

use async_trait::async_trait;

use crate::error::NotifyError;
use crate::product::{Product, ProductDetails};
use crate::Notifier;

pub mod bestbuy;
pub mod evga;
pub mod newegg;

#[async_trait]
pub trait ScrapingProvider<'a> {
    async fn get_request(
        &'a self,
        details: &'a ProductDetails,
    ) -> Result<reqwest::Response, NotifyError> {
        reqwest::get(&details.page)
            .await
            .map_err(NotifyError::WebRequestFailed)
    }
    async fn handle_response(
        &'a self,
        resp: reqwest::Response,
        details: &'a ProductDetails,
    ) -> Result<Product, NotifyError>;

    async fn is_available(&'a self, details: &'a ProductDetails) -> Result<Product, NotifyError> {
        let resp = self.get_request(details).await?;
        let status = resp.status();

        if !status.is_success() || status.is_server_error() {
            return Err(NotifyError::NoPage);
        }

        if status.is_client_error() {
            return Err(NotifyError::WebClientError);
        }

        self.handle_response(resp, details).await
    }
}

pub async fn get_providers_from_scraping(
    notifier: &mut Notifier,
) -> Result<HashSet<Product>, NotifyError> {
    let mut futs = vec![];
    for page in &notifier.config.products {
        futs.push(page.is_available());
    }

    let joined = futures::future::join_all(futs).await;

    let mut providers = HashSet::new();
    for res in joined {
        match res {
            Ok(res) => {
                if !providers.insert(res) {
                    eprintln!("Duplicate provider found.");
                }
            }
            Err(NotifyError::WebRequestFailed(e)) => eprintln!("{}", e),
            _ => {}
        }
    }

    Ok(providers)
}
