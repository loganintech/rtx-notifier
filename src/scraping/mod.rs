use std::collections::HashSet;

use async_trait::async_trait;
use chrono::{Local, Duration};

use crate::error::NotifyError;
use crate::Notifier;
use crate::product::Product;
use reqwest::{StatusCode, header::HeaderMap};

pub mod bestbuy;
pub mod evga;
pub mod newegg;
pub mod nvidia;
pub mod bnh;

#[async_trait]
pub trait ScrapingProvider<'a> {
    async fn get_request(
        &'a self,
        product: &'a Product,
    ) -> Result<reqwest::Response, NotifyError> {
        // Create a new client, can't use the reqwest::get() because we need headers
        let client = reqwest::Client::new();
        let mut headers = HeaderMap::new();
        // Add some headers for a user agent, otherwise the host refuses connection
        headers.insert("User-Agent", "User-Agent: Mozilla/5.0 (Windows NT 10.0; Win64; x64; rv:80.0) Gecko/20100101 Firefox/80.0".parse().unwrap());

        // Load the webpage
        client
            .get(product.get_url()?)
            .headers(headers)
            .send()
            .await
            .map_err(NotifyError::WebRequestFailed)
    }
    async fn handle_response(
        &'a self,
        resp: reqwest::Response,
        details: &'a Product,
    ) -> Result<Product, NotifyError>;

    async fn is_available(&'a self, product: &'a Product) -> Result<Product, NotifyError> {
        let resp = self.get_request(product).await?;
        let status = resp.status();

        // If we're being rate limited
        if status == StatusCode::from_u16(429).unwrap() {
            return Err(NotifyError::RateLimit);
        }

        if !status.is_success() || status.is_server_error() {
            return Err(NotifyError::NoPage);
        }

        if status.is_client_error() {
            return Err(NotifyError::WebClientError);
        }

        self.handle_response(resp, product).await
    }
}

pub async fn get_providers_from_scraping(
    notifier: &mut Notifier,
) -> Result<HashSet<Product>, NotifyError> {
    if !notifier.config.application_config.should_scrape() {
        return Ok(HashSet::new());
    }

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
            Err(NotifyError::RateLimit) => notifier.config.application_config.scraping_timeout = Some(Local::now() + Duration::minutes(5)),
            Err(NotifyError::WebRequestFailed(e)) => eprintln!("{}", e),
            _ => {}
        }
    }

    Ok(providers)
}
