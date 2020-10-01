use async_trait::async_trait;
use lazy_static::lazy_static;
use regex::Regex;

use crate::{error::NotifyError, product::Product, scraping::ScrapingProvider};

lazy_static! {
    // Look for the javascript tag that loads the raw product.rs data from their webservers
    static ref DETAIL_REGEX: Regex =
        Regex::new(r#"<script type="text/javascript" src="(.+ItemInfo4.+)">"#).unwrap();
}

pub struct NeweggScraper;

#[async_trait]
impl<'a> ScrapingProvider<'a> for NeweggScraper {
    async fn handle_response(
        &'a self,
        resp: reqwest::Response,
        product: &'a Product,
    ) -> Result<Product, NotifyError> {
        let resp = resp
            .text()
            .await
            .map_err(|_| NotifyError::HTMLParseFailed)?;

        if resp.contains("We apologize for the confusion, but we can't quite tell if you're a person or a script.") {
            return Err(NotifyError::RateLimit);
        }

        let capture = DETAIL_REGEX.captures_iter(&resp).next();
        // If we found the js tag with the detail URL, act on it
        if let Some(capture) = capture {
            // Extract the URL knowing capture[0] is the entire match, not just the capturing group
            let product_url = &capture[1];

            // And load the product.rs url
            let product_resp = reqwest::get(product_url)
                .await
                .map_err(NotifyError::WebRequestFailed)?
                .text()
                .await
                .map_err(|_| NotifyError::HTMLParseFailed)?;

            // Then look for the JSON property that shows it's in stock. Yes, we could serialize this but why bother right now
            if product_resp.contains(r#""instock":true"#) {
                return Ok(product.clone());
            }
        }

        Err(NotifyError::NoProductFound)
    }
}
