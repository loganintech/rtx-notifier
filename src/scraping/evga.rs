use async_trait::async_trait;
use scraper::{Html, Selector};
use std::io::Write;

use crate::{error::NotifyError, product::Product, scraping::ScrapingProvider};

pub struct EvgaScraper;

#[async_trait]
impl<'a> ScrapingProvider<'a> for EvgaScraper {
    async fn handle_response(
        &'a self,
        resp: reqwest::Response,
        product: &'a Product,
    ) -> Result<Product, NotifyError> {
        let resp = resp
            .text()
            .await
            .map_err(|_| NotifyError::HTMLParseFailed)?;
        if resp.contains("There has been an error while requesting your page") {
            return Err(NotifyError::NoProductFound);
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
