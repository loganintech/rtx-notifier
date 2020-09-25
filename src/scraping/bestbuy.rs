use async_trait::async_trait;
use lazy_static::lazy_static;
use regex::{Regex, RegexBuilder};
use reqwest::header::HeaderMap;

use crate::{
    error::NotifyError,
    product::{Product, ProductDetails},
    scraping::ScrapingProvider,
};

// LOok for the div that says it's Sold Out, case insensitive. Give it a bit of before and after HTML so that it doesn't false match on other elements
lazy_static! {
    static ref BUTTON_REGEX: Regex =
        RegexBuilder::new(r#"<div class="fulfillment.+Sold Out</button></div></div>"#)
            .case_insensitive(true)
            .build()
            .expect("Invalid regex");
}

pub struct BestBuyScraper;

#[async_trait]
impl<'a> ScrapingProvider<'a> for BestBuyScraper {
    async fn get_request(
        &'a self,
        details: &'a ProductDetails,
    ) -> Result<reqwest::Response, NotifyError> {
        // Create a new client, can't use the reqwest::get() because we need headers
        let client = reqwest::Client::new();
        let mut headers = HeaderMap::new();
        // Add some headers for a user agent, otherwise the host refuses connection
        headers.insert("User-Agent", "User-Agent: Mozilla/5.0 (Windows NT 10.0; Win64; x64; rv:80.0) Gecko/20100101 Firefox/80.0".parse().unwrap());

        // Load the webpage
        client
            .get(&details.page)
            .headers(headers)
            .send()
            .await
            .map_err(NotifyError::WebRequestFailed)
    }

    async fn handle_response(
        &'a self,
        resp: reqwest::Response,
        details: &'a ProductDetails,
    ) -> Result<Product, NotifyError> {
        let resp = resp
            .text()
            .await
            .map_err(|_| NotifyError::HTMLParseFailed)?;
        // If we can't find the sold out button, we're back in stock
        if BUTTON_REGEX.captures_iter(&resp).next().is_none() {
            return Ok(Product::BestBuy(ProductDetails {
                product: details.product.clone(),
                page: details.page.clone(),
                ..ProductDetails::default()
            }));
        }

        Err(NotifyError::NoProductFound)
    }
}
