use std::io::prelude::*;

use async_trait::async_trait;
use flate2::read::GzDecoder;
use lazy_static::lazy_static;
use regex::{Regex, RegexBuilder};
use tokio::io::AsyncWriteExt;

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

        let bytes = resp.slice(..).to_vec();
        let mut decoder = GzDecoder::new(bytes.as_slice());
        let mut resp = String::new();

        if let Err(e) = decoder.read_to_string(&mut resp) {
            return Err(NotifyError::DecompressionError(e));
        }

        if !resp.contains(r#"Currently unavailable.</span>"#) && !OTHER_SELLER_REGEX.is_match(&resp)
        {
            let mut foptions = tokio::fs::OpenOptions::new();
            let foptions = foptions.create(true).write(true).append(true);
            let mut file = foptions
                .open(format!(
                    "{}-amazon-log.txt",
                    chrono::Local::now().to_rfc3339(),
                ))
                .await
                .map_err(NotifyError::FileIOError)?;
            file.write_all(resp.as_bytes())
                .await
                .map_err(NotifyError::FileIOError)?;

            return Ok(product.clone());
        }

        Err(NotifyError::NoProductFound)
    }
}
