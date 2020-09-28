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
        let headers = resp.headers().clone();
        let resp_bytes = resp
            .bytes()
            .await
            .map_err(|_| NotifyError::HTMLParseFailed)?;

        let bytes = resp_bytes.slice(..).to_vec();
        let mut decoder = GzDecoder::new(bytes.as_slice());
        let mut resp_decompressed = String::new();

        if let Err(e) = decoder.read_to_string(&mut resp_decompressed) {
            // If you know a better way of doing this block I'm all ears
            if let Some(content_type) = headers.get("Content-Type") {
                if let Ok("text/html") = content_type.to_str() {
                    if let Ok(page) = String::from_utf8(bytes.clone()) {
                        if page.contains(r#"<p class="a-last">Sorry, we just need to make sure you're not a robot. For best results, please make sure your browser is accepting cookies.</p>"#) {
                            return Err(NotifyError::RateLimit);
                        }
                    }
                }
            }
            return Err(NotifyError::DecompressionError(e));
        }

        if !resp_decompressed.contains(r#"Currently unavailable.</span>"#) && !OTHER_SELLER_REGEX.is_match(&resp_decompressed)
        {
            return Ok(product.clone());
        }

        Err(NotifyError::NoProductFound)
    }
}

// async fn write_amazon_response<'a, T: Into<&'a [u8]>>(resp: T) -> Result<(), NotifyError> {
//     let mut file = tokio::fs::File::create(format!(
//         "./amazon-log-{}.txt",
//         chrono::Local::now().to_rfc3339().replace(":", "-"),
//     ))
//     .await
//     .map_err(NotifyError::FileIOError)?;
//     file.write_all(resp.into())
//         .await
//         .map_err(NotifyError::FileIOError)?;
//
//     Ok(())
// }
