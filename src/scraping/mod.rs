use std::collections::HashSet;

use async_trait::async_trait;

use crate::error::NotifyError;
use crate::Notifier;
use crate::product::Product;

pub mod bestbuy;
pub mod newegg;
pub mod evga;

#[async_trait]
trait ScrapingProvider<'a> {
    fn get_endpoint(&'a self) -> &'a str;
    async fn handle_response(&self, resp: reqwest::Response) -> Result<Product, NotifyError>;
    async fn is_available(&'a self) -> Result<Product, NotifyError> {
        let resp = reqwest::get(self.get_endpoint())
            .await
            .map_err(NotifyError::WebRequestFailed)?;
        let status = resp.status();

        if !status.is_success() || status.is_server_error() {
            return Err(NotifyError::NoPage)
        }

        if status.is_client_error() {
            return Err(NotifyError::WebClientError)
        }
        self.handle_response(resp).await
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
            Ok(res) => if !providers.insert(res) { eprintln!("Duplicate provider found."); },
            Err(NotifyError::WebRequestFailed(e)) => eprintln!("{}", e),
            _ => {}
        }
    }

    Ok(providers)
}
