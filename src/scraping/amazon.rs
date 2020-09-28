use std::io::prelude::*;

use async_trait::async_trait;
use flate2::read::GzDecoder;
use lazy_static::lazy_static;
use regex::{Regex, RegexBuilder};

use crate::{error::NotifyError, product::Product, scraping::ScrapingProvider};

lazy_static! {
    // See if it's offering us a sale on another seller
    static ref OTHER_SELLER_REGEX: Regex =
        RegexBuilder::new("Available from .+these sellers</a>").case_insensitive(true).build().unwrap();
}

pub struct AmazonScraper;

#[async_trait]
impl<'a> ScrapingProvider<'a> for AmazonScraper {
    async fn handle_response(
        &'a self,
        resp: reqwest::Response,
        product: &'a Product,
    ) -> Result<Product, NotifyError> {
        let resp = resp
            .bytes()
            .await
            .map_err(|_| NotifyError::HTMLParseFailed)?;

        let bytes = resp.slice(0..).to_vec();
        let mut decoder = GzDecoder::new(bytes.as_slice());
        let mut resp = String::new();

        if let Err(e) = decoder.read_to_string(&mut resp) {
            return Err(NotifyError::DecompressionError(e));
        }

        if !resp
            .contains(r#"<span class="a-color-price a-text-bold">Currently unavailable.</span>"#)
            && !OTHER_SELLER_REGEX.is_match(&resp)
        {
            return Ok(product.clone());
        }

        Err(NotifyError::NoProductFound)
    }
}
