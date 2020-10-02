use async_trait::async_trait;
use scraper::{Html, Selector};
use std::io::Write;

use lazy_static::lazy_static;
use regex::{RegexBuilder, Regex};

lazy_static! {
    // Sometimes EVGA responds with a completely empty page that passes all of the request failure checks
    // Since it doesn't have the "not in stock" text, it is a false positive. But we don't want to allow these
    // ... empty pages to trigger the bot. So let's ignore them
    static ref EMPTY_PAGE_REGEX: Regex = RegexBuilder::new(r#"<html><head></head><body></body></html>"#)
        .ignore_whitespace(true).case_insensitive(true).build().unwrap();

    static ref ERROR_TITLE_REGEX: Regex = RegexBuilder::new(r#"<title>error</title>"#)
        .ignore_whitespace(true).case_insensitive(true).build().unwrap();
}

use crate::{error::NotifyError, product::Product, scraping::ScrapingProvider};

pub struct EvgaScraper;

#[async_trait]
impl<'a> ScrapingProvider<'a> for EvgaScraper {
    async fn handle_response(
        &'a self,
        resp: reqwest::Response,
        product: &'a Product,
    ) -> Result<Product, NotifyError> {
        let status = resp.status();
        let resp = resp
            .text()
            .await
            .map_err(|_| NotifyError::HTMLParseFailed)?;
        if resp.contains("There has been an error while requesting your page") {
            return Err(NotifyError::NoProductFound);
        }

        if EMPTY_PAGE_REGEX.is_match(resp.as_str()) || ERROR_TITLE_REGEX.is_match(resp.as_str()) {
            return Err(NotifyError::WebServer(status));
        }

        let document = Html::parse_document(&resp);

        let selector = Selector::parse("#LFrame_pnlOutOfStock")
            .map_err(|_| NotifyError::HTMLParseFailed)?;
        let mut selected = document.select(&selector);
        let found = selected.next();

        if found.is_none()
            || (found.is_some()
                && !found
                    .unwrap()
                    .inner_html()
                    .to_ascii_lowercase()
                    .contains("out of stock"))
        {
            let _ = write_evga_response(resp.as_bytes());
            return Ok(product.clone());
        }

        Err(NotifyError::NoProductFound)
    }
}

#[allow(dead_code)]
fn write_evga_response<'a, T: Into<&'a [u8]>>(resp: T) -> Result<(), NotifyError> {
    let mut file = std::fs::File::create(format!(
        "./evga_log/evga-log-{}.txt",
        chrono::Local::now().to_rfc3339().replace(":", "-"),
    ))
    .map_err(NotifyError::FileIOError)?;

    file.write_all(resp.into())
        .map_err(NotifyError::FileIOError)?;

    Ok(())
}
