use std::collections::{HashMap, HashSet};

use async_trait::async_trait;
use chrono::{Duration, Local};

use crate::error::NotifyError;
use crate::product::Product;
use crate::Notifier;

pub mod amazon;
pub mod bestbuy;
pub mod bnh;
pub mod evga;
pub mod newegg;
pub mod nvidia;

#[async_trait]
pub trait ScrapingProvider<'a> {
    async fn get_request(&'a self, product: &'a Product) -> Result<reqwest::Response, NotifyError> {
        // Create a new client, can't use the reqwest::get() because we need headers
        let client = reqwest::ClientBuilder::new()
            .user_agent(
                "Mozilla/5.0 (Windows NT 10.0; Win64; x64; rv:80.0) Gecko/20100101 Firefox/80.0",
            )
            .gzip(true)
            .build()
            .map_err(|_| NotifyError::ClientBuild)?;

        // Load the webpage
        client
            .get(product.get_url()?)
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
        //https://developer.mozilla.org/en-US/docs/Web/HTTP/Status/429
        if status.as_u16() == 429
            || (product.to_key() == "bnh"
                && resp.url().as_str() == "https://site-not-available.bhphotovideo.com/500Error")
        {
            return Err(NotifyError::RateLimit);
        }

        if status.is_server_error() {
            return Err(NotifyError::WebServer(status));
        }

        if status.is_client_error() {
            return Err(NotifyError::WebClient(status));
        }

        if !status.is_success() {
            return Err(NotifyError::BadStatus(status));
        }

        self.handle_response(resp, product).await
    }
}

pub async fn get_providers_from_scraping(
    notifier: &mut Notifier,
) -> Result<HashSet<Product>, NotifyError> {
    let mut futs = vec![];
    let active_products = notifier
        .config
        .products
        .iter()
        .filter(|p| p.is_active() && notifier.config.application_config.should_scrape(p.to_key()))
        .collect::<Vec<&Product>>();

    for product in &active_products {
        futs.push(product.is_available());
    }

    let joined = futures::future::join_all(futs).await;

    let mut checked: HashMap<&str, (usize, Vec<&str>)> = HashMap::new();
    let mut providers = HashSet::new();
    for (i, res) in joined.into_iter().enumerate() {
        let product = active_products[i];
        match res {
            Ok(res) => {
                checked
                    .entry(product.to_key())
                    .and_modify(|(count, products)| { *count += 1; products.push(product.get_name().unwrap_or("")) })
                    .or_insert((1, vec![product.get_name().unwrap_or("")]));
                if !providers.insert(res) {
                    eprintln!("Duplicate provider found.");
                }
            }
            Err(NotifyError::RateLimit) => {
                println!("Rate Limiting Hit: {:?}", product);
                if notifier.config.application_config.ratelimit_keys.is_some() {
                    notifier
                        .config
                        .application_config
                        .ratelimit_keys
                        .as_mut()
                        .unwrap()
                        .insert(
                            product.to_key().to_string(),
                            Local::now() + Duration::minutes(2),
                        );
                } else {
                    let mut map = std::collections::HashMap::new();
                    map.insert(
                        product.to_key().to_string(),
                        Local::now() + Duration::minutes(2),
                    );
                    notifier.config.application_config.ratelimit_keys = Some(map);
                }
            }
            Err(NotifyError::WebRequestFailed(e)) => eprintln!("{}", e),
            Err(NotifyError::NoProductFound) => {
                checked
                    .entry(product.to_key())
                    .and_modify(|(count, products)| { *count += 1; products.push(product.get_name().unwrap_or("")) })
                    .or_insert((1, vec![product.get_name().unwrap_or("")]));
            }
            Err(e) => eprintln!(
                "==========\nError Happened: {}\n====\nWith Product: {:?}\n==========",
                e, product
            ),
        }
    }

    println!("Sites Checked:");
    for (key, (count, list)) in checked.keys().zip(checked.values()) {
        println!("[{:02}] {}: {:?}", count, key, list);
    }
    // println!("Sites Checked: {:#?}", checked);

    Ok(providers)
}
