use async_trait::async_trait;
use scraper::{Html, Selector};

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

        let selector = Selector::parse(&product.get_css_selector()?)
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
            return Ok(product.clone());
        }

        Err(NotifyError::NoProductFound)
    }
}
