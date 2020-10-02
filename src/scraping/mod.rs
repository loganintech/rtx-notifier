use std::collections::{HashMap, HashSet};

use async_trait::async_trait;

use crate::error::NotifyError;
use crate::product::Product;
use crate::Notifier;
use futures::TryFutureExt;

pub mod amazon;
pub mod bestbuy;
pub mod bnh;
pub mod evga;
pub mod newegg;
pub mod nvidia;



#[async_trait]
pub trait ScrapingProvider<'a> {
    async fn get_request(
        &'a self,
        product: &'a Product,
        client: &reqwest::Client,
    ) -> Result<reqwest::Response, NotifyError> {
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

    async fn is_available(
        &'a self,
        product: &'a Product,
        client: &reqwest::Client,
    ) -> Result<Product, NotifyError> {
        let resp = self.get_request(product, client).await?;
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

fn get_client(notifier: &Notifier) -> Result<reqwest::Client, NotifyError> {
    let proxy_url = &notifier.config.application_config.proxy_url;
    let mut client_builder = reqwest::ClientBuilder::new()
        .user_agent(
            "Mozilla/5.0 (Windows NT 10.0; Win64; x64; rv:80.0) Gecko/20100101 Firefox/80.0",
        )
        .gzip(true);
    if let Some(proxy_url) = proxy_url {
        let proxy = reqwest::Proxy::all(proxy_url).map_err(|_| NotifyError::ProxyNotRunning)?;
        client_builder = client_builder.proxy(proxy);
    }
    let client = client_builder
        .build()
        .map_err(|_| NotifyError::ClientBuild)?;

    Ok(client)
}

pub async fn get_providers_from_scraping(
    notifier: &mut Notifier,
) -> Result<HashSet<Product>, NotifyError> {
    let client = get_client(&notifier)?;
    let active_products = notifier
        .config
        .products
        .iter()
        .filter(|p| p.is_active() && notifier.config.application_config.should_scrape(p.to_key()))
        .cloned()
        .collect::<Vec<Product>>();

    let mut futs = vec![];
    for product in &active_products {
        futs.push(product.is_available(&client));
    }

    let joined = futures::future::join_all(futs).await;

    let mut should_reload_tor = false;
    let mut checked: HashMap<&str, (usize, Vec<String>)> = HashMap::new();
    let mut providers = HashSet::new();
    for (i, res) in joined.into_iter().enumerate() {
        let product = &active_products[i];
        match res {
            Ok(res) => {
                modify_checked_map(product, &mut checked);
                if !providers.insert(res) {
                    eprintln!("Duplicate provider found.");
                }
            }
            Err(NotifyError::RateLimit) => {
                should_reload_tor = true;
                print_err(product, NotifyError::RateLimit);
                notifier.add_ratelimit(&product);
            }
            Err(NotifyError::WebRequestFailed(e)) => print_err(product, e),
            Err(NotifyError::NoProductFound) => modify_checked_map(product, &mut checked),
            Err(e) => print_err(product, e),
        }
    }

    #[cfg(target_os="linux")]
    if should_reload_tor {
        if let Err(e) = reload_tor() {
            eprintln!("Error reloading TOR: {}", e);
        }
    }

    println!("Sites Checked:");
    for (key, (count, list)) in checked.keys().zip(checked.values()) {
        println!("[{:02}] {}: {:?}", count, key, list);
    }


    Ok(providers)
}

fn print_err(product: &Product, e: impl std::error::Error) {
    eprintln!(
        "==========\nError Happened: {}\n====\nWith Product: {:?}\n==========",
        e, product
    );
}

fn modify_checked_map(product: &Product, map: &mut HashMap<&str, (usize, Vec<String>)>) {
    let name = product.get_name().unwrap_or("").to_string();
    map.entry(product.to_key())
        .and_modify(|(count, products)| {
            *count += 1;
            products.push(name.clone())
        })
        .or_insert((1, vec![name]));
}

#[cfg(target_os="linux")]
fn reload_tor() -> Result<(), NotifyError> {
    let mut child = std::process::Command::new("service")
        .args(&["tor", "reload"])
        .spawn()
        .map_err(NotifyError::CommandErr)?;
    let res = child.wait().map_err(NotifyError::CommandErr)?;
    if res.success() {
        Ok(())
    } else {
        Err(NotifyError::CommandResult(res.code().unwrap_or(0)))
    }
}
